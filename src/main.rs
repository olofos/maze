use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin};
use bevy::time::common_conditions::on_timer;

use bevy::window::PresentMode;
#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use tilemap::Tilemap;

mod components;
mod consts;
mod maze;
mod tilemap;
mod tileset;

use crate::components::*;
use crate::consts::*;

#[allow(dead_code)]
#[derive(States, Debug, Default, Clone, PartialEq, Eq, Hash)]
enum GameState {
    MainMenu,
    #[default]
    GeneratingMaze,
    Playing,
    LevelDone,
}

// Background color: b28d70

fn main() {
    let mut app = App::new();

    let present_mode = if cfg!(target_arch = "wasm32") {
        PresentMode::default()
    } else {
        PresentMode::Immediate
    };

    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Maze".to_string(),
                resizable: false,
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                present_mode,
                ..default()
            }),
            ..default()
        }),
        Material2dPlugin::<TilemapMaterial>::default(),
    ))
    .insert_state(GameState::default())
    .add_systems(FixedUpdate, tilemap::update_tilemaps)
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
    .add_systems(FixedUpdate, construct_tilemap)
    .add_systems(FixedUpdate, maze::update_tilemap)
    // semicolon
    ;
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
        handle: asset_server.load("tileset4.png"),
    });
}

fn construct_tilemap(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TilesetHandle)>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok((entity, handle)) = query.get_single_mut() else {
        return;
    };
    let handle = handle.handle.clone();

    let Some(image) = images.get(&handle) else {
        return;
    };

    let tileset_image = tileset::expand(image);

    commands.spawn(Tilemap::new(
        images.add(tileset_image),
        GRID_WIDTH as u32,
        GRID_HEIGHT as u32,
        5.0,
    ));

    commands.entity(entity).despawn();
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TilemapMaterial {
    #[uniform(0)]
    grid_size: Vec4,
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
        "shaders/tilemap.wgsl".into()
    }
}

fn setup_player_and_goal(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("elephant-square.png"),

            transform: Transform {
                translation: Vec3::new(
                    (GRID_WIDTH - 1) as f32 + 0.5,
                    (GRID_HEIGHT - 1) as f32 + 0.5,
                    1.,
                ),
                scale: Vec3::new(1.0 / 192.0, 1.0 / 192.0, 1.0),
                ..default()
            },
            ..default()
        },
        Goal,
        Name::from("Goal"),
    ));

    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("elephant-round.png"),
            transform: Transform {
                translation: Vec3::new(0.5, 0.5, 2.),
                scale: Vec3::new(PLAYER_WIDTH / 192.0, PLAYER_HEIGHT / 192.0, 1.0),
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

    let min_x = if is_between.y || walls.e { pos.x } else { 0.0 };
    let max_x = if is_between.y || walls.w {
        pos.x + 1.0
    } else {
        GRID_WIDTH as f32
    };
    let min_y = if is_between.x || walls.s { pos.y } else { 0.0 };
    let max_y = if is_between.x || walls.n {
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
