use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat,
};
use bevy::render::texture::ImageSampler;
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use bevy::time::common_conditions::on_timer;

#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

mod components;
mod consts;
mod maze;

use crate::components::*;
use crate::consts::*;

#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    MainMenu,
    GeneratingMaze,
    Playing,
    LevelDone,
}

fn main() {
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Maze".to_string(),
                resizable: false,
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }),
        Material2dPlugin::<CustomMaterial>::default(),
    ))
    .insert_state(GameState::default())
    .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
    .add_systems(Startup, setup)
    .add_systems(OnEnter(GameState::Playing), setup_player_and_goal)
    .add_systems(OnEnter(GameState::GeneratingMaze), maze::setup)
    .add_systems(Update, close_on_esc)
    .add_systems(
        FixedUpdate,
        move_player.run_if(in_state(GameState::Playing)),
    )
    .add_systems(
        Update,
        maze::generate
            .run_if(in_state(GameState::GeneratingMaze))
            .run_if(on_timer(Duration::from_millis(MAZE_GEN_TIME_MS))),
    )
    .add_systems(FixedUpdate, check_goal.run_if(in_state(GameState::Playing)))
    .add_systems(FixedUpdate, construct_tilemap);
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(WorldInspectorPlugin::new().run_if(
        bevy::input::common_conditions::input_toggle_active(false, KeyCode::Backquote),
    ));

    app.run();
}

fn collide(transform1: &Transform, transform2: &Transform) -> bool {
    (transform1.translation.xy() - transform2.translation.xy())
        .abs()
        .cmplt((transform1.scale.xy() + transform2.scale.xy()) / 2.0)
        .all()
}

fn check_goal(
    mut ev_appexit: EventWriter<AppExit>,
    player_query: Query<&Transform, (With<Player>, Without<Goal>)>,
    goal_query: Query<&Transform, (Without<Player>, With<Goal>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let player_transform = player_query.single();
    let goal_transform = goal_query.single();

    if collide(player_transform, goal_transform) {
        next_state.set(GameState::LevelDone);
        ev_appexit.send(AppExit::Success);
    }
}

fn create_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![
            [0.0, 0.0, 0.0],
            [0.0, 16.0, 0.0],
            [16.0, 16.0, 0.0],
            [16.0, 0.0, 0.0],
        ],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, {
        vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
    })
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 2, 3, 0]))
}

#[derive(Component)]
struct TilemapHandle {
    handle: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    images: Res<Assets<Image>>,
) {
    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(GRID_WIDTH as f32 / 2., GRID_HEIGHT as f32 / 2., 0.0),
            scale: Vec3::new(PIXEL_WIDTH, PIXEL_HEIGHT, 1.0),
            ..default()
        },
        ..default()
    });

    let texture_handle = asset_server.load("tileset3.png");

    println!("Texture handle: {:?}", texture_handle);
    println!("Image: {:?}", images.get(&texture_handle));

    commands.spawn(TilemapHandle {
        handle: texture_handle.clone(),
    });

    // let mesh_handle = meshes.add(create_mesh());
    // commands.spawn((MaterialMesh2dBundle {
    //     mesh: mesh_handle.into(),
    //     transform: Transform::default().with_translation(Vec3::new(0.0, 0.0, 4.0)),
    //     material: materials.add(CustomMaterial {
    //         color: LinearRgba::BLUE,
    //         color_texture: Some(texture_handle),
    //     }),
    //     ..default()
    // },));
}

fn blit_tile(src: &[u8], dst: &mut [u8], tile_num: usize, pos: (usize, usize), width: usize) {
    const SUBTILE_WIDTH: usize = 16;
    const SUBTILE_HEIGHT: usize = 16;
    const CHANNELS: usize = 4;

    let (pos_x, pos_y) = pos;
    let src = &src[(tile_num) * (CHANNELS * SUBTILE_WIDTH * SUBTILE_HEIGHT)
        ..(tile_num + 1) * (CHANNELS * SUBTILE_WIDTH * SUBTILE_HEIGHT)];
    let dst = &mut dst[(pos_y * CHANNELS * width + CHANNELS * pos_x)..];

    for y in 0..SUBTILE_HEIGHT {
        for x in 0..SUBTILE_WIDTH {
            for i in 0..CHANNELS {
                dst[CHANNELS * width * y + CHANNELS * x + i] =
                    src[(CHANNELS * SUBTILE_WIDTH * (SUBTILE_HEIGHT - y - 1)) + CHANNELS * x + i];
            }
        }
    }
}

#[derive(Clone, Copy)]
enum SubTile {
    Full = 0,
    N = 1,
    E = 2,
    S = 3,
    W = 4,
    SW = 5,
    NW = 6,
    NE = 7,
    SE = 8,
    CornerNW = 9,
    CornerNE = 10,
    CornerSE = 11,
    CornerSW = 12,
    Empty = 13,
}

fn construct_tilemap(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    query: Query<(Entity, &TilemapHandle)>,
    mut images: ResMut<Assets<Image>>,
) {
    if let Ok((entity, handle)) = query.get_single() {
        let handle = handle.handle.clone();

        if let Some(image) = images.get_mut(&handle) {
            commands.entity(entity).despawn();
            let mut tile = vec![0u8; 64 * 64 * 16 * 4];
            // for y in 0..4 {
            //     for x in 0..4 {
            //         blit_tile(&image.data, &mut tile, 4 * y + x, (x * 16, y * 16), 64);
            //     }
            // }

            use SubTile::*;

            let tiles_0 = [
                [Full, Full, Full, Full],
                [Full, Full, Full, Full],
                [Full, Full, Full, Full],
                [Full, Full, Full, Full],
            ];

            let tiles_1 = [
                [W, Empty, Empty, E],
                [W, Empty, Empty, E],
                [W, Empty, Empty, E],
                [SW, S, S, SE],
            ];

            let tiles_2 = [
                [NW, N, N, N],
                [W, Empty, Empty, Empty],
                [W, Empty, Empty, Empty],
                [SW, S, S, S],
            ];

            let tiles_3 = [
                [W, Empty, Empty, CornerNE],
                [W, Empty, Empty, Empty],
                [W, Empty, Empty, Empty],
                [SW, S, S, S],
            ];

            let tiles_4 = [
                [NW, N, N, NE],
                [W, Empty, Empty, E],
                [W, Empty, Empty, E],
                [W, Empty, Empty, E],
            ];

            let tiles_5 = [
                [W, Empty, Empty, E],
                [W, Empty, Empty, E],
                [W, Empty, Empty, E],
                [W, Empty, Empty, E],
            ];

            let tiles_6 = [
                [NW, N, N, N],
                [W, Empty, Empty, Empty],
                [W, Empty, Empty, Empty],
                [W, Empty, Empty, CornerSE],
            ];

            let tiles_7 = [
                [W, Empty, Empty, CornerNE],
                [W, Empty, Empty, Empty],
                [W, Empty, Empty, Empty],
                [W, Empty, Empty, CornerSE],
            ];

            let tiles_8 = [
                [N, N, N, NE],
                [Empty, Empty, Empty, E],
                [Empty, Empty, Empty, E],
                [S, S, S, SE],
            ];

            let tiles_9 = [
                [CornerNW, Empty, Empty, E],
                [Empty, Empty, Empty, E],
                [Empty, Empty, Empty, E],
                [S, S, S, SE],
            ];

            let tiles_10 = [
                [N, N, N, N],
                [Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty],
                [S, S, S, S],
            ];

            let tiles_11 = [
                [CornerNW, Empty, Empty, CornerNE],
                [Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty],
                [S, S, S, S],
            ];

            let tiles_12 = [
                [N, N, N, NE],
                [Empty, Empty, Empty, E],
                [Empty, Empty, Empty, E],
                [CornerSW, Empty, Empty, E],
            ];

            let tiles_13 = [
                [CornerNW, Empty, Empty, E],
                [Empty, Empty, Empty, E],
                [Empty, Empty, Empty, E],
                [CornerSW, Empty, Empty, E],
            ];

            let tiles_14 = [
                [N, N, N, N],
                [Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty],
                [CornerSW, Empty, Empty, CornerSE],
            ];

            let tiles_15 = [
                [CornerNW, Empty, Empty, CornerNE],
                [Empty, Empty, Empty, Empty],
                [Empty, Empty, Empty, Empty],
                [CornerSW, Empty, Empty, CornerSE],
            ];

            for (i, data) in [
                tiles_0, tiles_1, tiles_2, tiles_3, tiles_4, tiles_5, tiles_6, tiles_7, tiles_8,
                tiles_9, tiles_10, tiles_11, tiles_12, tiles_13, tiles_14, tiles_15,
            ]
            .iter()
            .enumerate()
            {
                for y in 0..4 {
                    for x in 0..4 {
                        blit_tile(
                            &image.data,
                            &mut tile[i * 64 * 64 * 4..],
                            data[4 - y - 1][x] as usize,
                            (x * 16, y * 16),
                            64,
                        );
                    }
                }
            }

            let mut tile_image = Image::new(
                Extent3d {
                    width: 64,
                    height: 64 * 16,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                tile,
                image.texture_descriptor.format,
                RenderAssetUsages::all(),
            );
            tile_image.sampler = ImageSampler::nearest();

            let tilemap_buf = vec![3, 11, 9, 1, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4];

            let mut tilemap_image = Image::new(
                Extent3d {
                    width: 4,
                    height: 4,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                tilemap_buf,
                TextureFormat::R8Unorm,
                RenderAssetUsages::all(),
            );
            tilemap_image.sampler = ImageSampler::nearest();

            let image_handle = images.add(tile_image);
            let tilemap_handle = images.add(tilemap_image);
            let mesh_handle = meshes.add(create_mesh());
            commands.spawn((MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                transform: Transform::default().with_translation(Vec3::new(0.0, 0.0, 4.0)),
                material: materials.add(CustomMaterial {
                    tile_index: 7.0,
                    tileset_texture: Some(image_handle),
                    tilemap_texture: Some(tilemap_handle),
                }),
                ..default()
            },));
        }
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[uniform(0)]
    tile_index: f32,
    #[texture(1)]
    #[sampler(2)]
    tileset_texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    tilemap_texture: Option<Handle<Image>>,
}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for CustomMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material_2d.wgsl".into()
    }
}

fn setup_player_and_goal(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0., 0.7, 0.3),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(
                    (GRID_WIDTH - 1) as f32 + 0.5,
                    (GRID_HEIGHT - 1) as f32 + 0.5,
                    1.,
                ),
                scale: Vec3::new(0.75, 0.75, 1.0),
                ..default()
            },
            ..default()
        },
        Goal,
        Name::from("Goal"),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.0, 0.3, 0.7),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.5, 0.5, 2.),
                scale: Vec3::new(PLAYER_WIDTH, PLAYER_HEIGHT, 1.0),
                ..default()
            },
            ..default()
        },
        Name::from("Player"),
        Player,
    ));
}

fn move_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    grid_query: Query<&Grid>,
    time: Res<Time>,
) {
    let mut player_transform = player_query.single_mut();
    let grid = grid_query.single();

    let mut direction = Vec3::new(0., 0., 0.);

    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.;
    }

    direction *= PLAYER_SPEED * time.delta_seconds();

    if keyboard_input.pressed(KeyCode::ControlLeft) {
        direction *= 4.0;
    }

    let pos = player_transform.translation.xy().floor();
    let walls = &grid.walls[(pos.y * GRID_WIDTH as f32 + pos.x) as usize];

    let is_between = (player_transform.translation.xy() - (pos + Vec2::new(0.5, 0.5)))
        .abs()
        .cmpgt(Vec2::new(
            (1.0 - PLAYER_WIDTH) / 2.0,
            (1.0 - PLAYER_HEIGHT) / 2.0,
        ));

    let min_x = if is_between.y || walls.left.is_some() {
        pos.x
    } else {
        0.0
    };
    let max_x = if is_between.y || walls.right.is_some() {
        pos.x + 1.0
    } else {
        GRID_WIDTH as f32
    };
    let min_y = if is_between.x || walls.down.is_some() {
        pos.y
    } else {
        0.0
    };
    let max_y = if is_between.x || walls.up.is_some() {
        pos.y + 1.0
    } else {
        GRID_HEIGHT as f32
    };

    let d = Vec3::new(
        PLAYER_WIDTH / 2.0 + PIXEL_WIDTH,
        PLAYER_HEIGHT / 2.0 + PIXEL_HEIGHT,
        0.0,
    );

    player_transform.translation = (player_transform.translation + direction).clamp(
        Vec3::new(min_x, min_y, 0.) + d,
        Vec3::new(max_x, max_y, 0.) - d,
    );
}

pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}
