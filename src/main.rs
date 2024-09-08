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
    MainMenu,
    #[default]
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
        Material2dPlugin::<TilemapMaterial>::default(),
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
    let x = GRID_WIDTH as f32;
    let y = GRID_HEIGHT as f32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0, 0.0, 0.0], [0.0, y, 0.0], [x, y, 0.0], [x, 0.0, 0.0]],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, {
        vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
    })
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 2, 3, 0]))
}

#[derive(Component)]
struct TilesetHandle {
    handle: Handle<Image>,
}

#[derive(Component)]
struct TilemapHandle {
    material: Handle<TilemapMaterial>,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(GRID_WIDTH as f32 / 2., GRID_HEIGHT as f32 / 2., 0.0),
            scale: Vec3::new(PIXEL_WIDTH, PIXEL_HEIGHT, 1.0),
            ..default()
        },
        ..default()
    });

    commands.spawn(TilesetHandle {
        handle: asset_server.load("tileset3.png"),
    });
}

const SUBTILE_WIDTH: usize = 16;
const SUBTILE_HEIGHT: usize = 16;
const CHANNELS: usize = 4;
const TILE_WIDTH: usize = PLAYFIELD_WIDTH as usize / GRID_WIDTH / 2;
const TILE_HEIGHT: usize = PLAYFIELD_HEIGHT as usize / GRID_HEIGHT / 2;
const NUM_TILES: usize = 16;

fn blit_tile(src: &[u8], dst: &mut [u8], tile_num: usize, pos: (usize, usize), width: usize) {
    let (pos_x, pos_y) = pos;
    let src = &src[(tile_num) * (CHANNELS * SUBTILE_WIDTH * SUBTILE_HEIGHT)
        ..(tile_num + 1) * (CHANNELS * SUBTILE_WIDTH * SUBTILE_HEIGHT)];
    let dst =
        &mut dst[(pos_y * SUBTILE_HEIGHT * CHANNELS * width + CHANNELS * SUBTILE_WIDTH * pos_x)..];

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
    mut query: Query<(Entity, &mut TilesetHandle)>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok((entity, handle)) = query.get_single_mut() else {
        return;
    };
    let handle = handle.handle.clone();

    let Some(image) = images.get_mut(&handle) else {
        return;
    };

    commands.entity(entity).despawn();
    let mut tile = vec![0u8; TILE_WIDTH * TILE_HEIGHT * NUM_TILES * CHANNELS];

    use SubTile::*;

    let max_x = TILE_WIDTH / SUBTILE_WIDTH - 1;
    let max_y = TILE_HEIGHT / SUBTILE_HEIGHT - 1;

    for x in 0..=max_x {
        for y in 0..=max_y {
            blit_tile(
                &image.data,
                &mut tile[0 * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
                Full as usize,
                (x, y),
                TILE_WIDTH,
            );
        }
    }

    for tile_num in 1..NUM_TILES {
        let ne = match tile_num & 0b0011 {
            0b0000 => NE,
            0b0001 => E,
            0b0010 => N,
            0b0011 => CornerNE,
            _ => unreachable!(),
        };
        let se = match tile_num & 0b0110 {
            0b0000 => SE,
            0b0010 => S,
            0b0100 => E,
            0b0110 => CornerSE,
            _ => unreachable!(),
        };
        let sw = match tile_num & 0b1100 {
            0b0000 => SW,
            0b0100 => W,
            0b1000 => S,
            0b1100 => CornerSW,
            _ => unreachable!(),
        };
        let nw = match tile_num & 0b1001 {
            0b0000 => NW,
            0b1000 => N,
            0b0001 => W,
            0b1001 => CornerNW,
            _ => unreachable!(),
        };
        let n = if (tile_num & 0b0001) == 0b0001 {
            Empty
        } else {
            N
        };
        let e = if (tile_num & 0b0010) == 0b0010 {
            Empty
        } else {
            E
        };
        let s = if (tile_num & 0b0100) == 0b0100 {
            Empty
        } else {
            S
        };
        let w = if (tile_num & 0b1000) == 0b1000 {
            Empty
        } else {
            W
        };

        blit_tile(
            &image.data,
            &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
            sw as usize,
            (0, 0),
            TILE_WIDTH,
        );

        blit_tile(
            &image.data,
            &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
            se as usize,
            (max_x, 0),
            TILE_WIDTH,
        );

        blit_tile(
            &image.data,
            &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
            nw as usize,
            (0, max_y),
            TILE_WIDTH,
        );

        blit_tile(
            &image.data,
            &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
            ne as usize,
            (max_x, max_y),
            TILE_WIDTH,
        );

        for x in 1..max_x {
            blit_tile(
                &image.data,
                &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
                s as usize,
                (x, 0),
                TILE_WIDTH,
            );

            blit_tile(
                &image.data,
                &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
                n as usize,
                (x, max_y),
                TILE_WIDTH,
            );
        }

        for y in 1..max_y {
            blit_tile(
                &image.data,
                &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
                w as usize,
                (0, y),
                TILE_WIDTH,
            );

            blit_tile(
                &image.data,
                &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
                e as usize,
                (max_x, y),
                TILE_WIDTH,
            );
        }

        for x in 1..max_x {
            for y in 1..max_y {
                blit_tile(
                    &image.data,
                    &mut tile[tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..],
                    Empty as usize,
                    (x, y),
                    TILE_WIDTH,
                );
            }
        }
    }

    let mut tileset_image = Image::new(
        Extent3d {
            width: TILE_WIDTH as u32,
            height: (TILE_HEIGHT * NUM_TILES) as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        tile,
        image.texture_descriptor.format,
        RenderAssetUsages::all(),
    );
    tileset_image.sampler = ImageSampler::nearest();
    tileset_image.reinterpret_stacked_2d_as_array(NUM_TILES as u32);

    let tilemap_buf = vec![0; GRID_WIDTH * GRID_HEIGHT];

    let mut tilemap_image = Image::new(
        Extent3d {
            width: GRID_WIDTH as u32,
            height: GRID_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        tilemap_buf,
        TextureFormat::R8Uint,
        RenderAssetUsages::all(),
    );
    tilemap_image.sampler = ImageSampler::nearest();

    let image_handle = images.add(tileset_image);
    let tilemap_handle = images.add(tilemap_image);
    let mesh_handle = meshes.add(create_mesh());

    let material = materials.add(TilemapMaterial {
        grid_size: Vec2::new(GRID_WIDTH as f32, GRID_HEIGHT as f32),
        tileset_texture: Some(image_handle),
        tilemap_texture: Some(tilemap_handle.clone()),
    });

    commands.spawn(TilemapHandle {
        material: material.clone(),
    });

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            transform: Transform::default().with_translation(Vec3::new(0.0, 0.0, -5.0)),
            material,
            ..default()
        },
        // TileMap,
    ));
}

#[derive(Component)]
struct TileMap;

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TilemapMaterial {
    #[uniform(0)]
    grid_size: Vec2,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    tileset_texture: Option<Handle<Image>>,
    #[texture(3, sample_type = "u_int")]
    tilemap_texture: Option<Handle<Image>>,
}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for TilemapMaterial {
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
