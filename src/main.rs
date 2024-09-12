use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

use bevy::window::PresentMode;
#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;
use states::{AppState, GamePlayState};
use tilemap::Tilemap;

mod components;
mod consts;
mod maze;
mod states;
mod tilemap;
mod tileset_builder;

use crate::components::*;
use crate::consts::*;

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
        tilemap::plugin,
        states::plugin,
        ))
    .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
    .register_type::<components::Grid>()
    .add_systems(Startup, setup)
    .add_systems(OnEnter(GamePlayState::GeneratingMaze), maze::setup)
    .add_systems(OnEnter(GamePlayState::Playing), setup_player_and_goal)
    .add_systems(Update, (
        tileset_builder::construct_tilemap,
        maze::generate
        .run_if(on_timer(Duration::from_millis(MAZE_GEN_TIME_MS))),
        
        generate_bg
    ).run_if(in_state(GamePlayState::GeneratingMaze)))
    .add_systems(Update, maze::update_tilemap.run_if(in_state(AppState::InGame)))
    .add_systems(
        Update,
        (move_player, check_goal).run_if(in_state(GamePlayState::Playing)),
    )
    // semicolon
    ;
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.add_plugins(WorldInspectorPlugin::new().run_if(
            bevy::input::common_conditions::input_toggle_active(false, KeyCode::Backquote),
        ))
        .add_systems(Update, close_on_esc);
    }

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
    mut next_state: ResMut<NextState<GamePlayState>>,
) {
    let player_transform = player_query.single();
    let goal_transform = goal_query.single();

    if collide(player_transform, goal_transform) {
        next_state.set(GamePlayState::LevelDone);
        ev_appexit.send(AppExit::Success);
    }
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

    commands.spawn((
        tileset_builder::Tileset {
            tileset: asset_server.load("tileset4.png"),
            grid_size: (GRID_WIDTH as u32, GRID_HEIGHT as u32),
        },
        Transform::default().with_translation(Vec3::new(0.0, 0.0, 5.0)),
        Trees,
        Name::from("Tilemap: Trees"),
    ));

    commands.spawn((
        tilemap::Tileset {
            image: asset_server.load("bg.png"),
            num_tiles: 7,
        },
        Tilemap::new(32, 32),
        Transform::default().with_translation(Vec3::new(0.0, 0.0, -5.0)),
        Ground,
        Name::from("Tilemap: Background"),
    ));
}

#[derive(Component)]
struct Trees;

#[derive(Component)]
struct Ground;

fn generate_bg(mut commands: Commands, mut query: Query<(Entity, &mut Tilemap), With<Ground>>) {
    let Ok((entity, mut tilemap)) = query.get_single_mut() else {
        return;
    };

    let width = tilemap.grid_size.x;
    let height = tilemap.grid_size.y;

    let mut rng = rand::thread_rng();

    for x in 0..width {
        for y in 0..height {
            tilemap.data[(y * width + x) as usize] = rng.gen_range(-17i32..7).clamp(0, 7) as u8;
        }
    }

    commands.entity(entity).remove::<Ground>();
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

    let min_x = if is_between.y || walls.w { pos.x } else { 0.0 };
    let max_x = if is_between.y || walls.e {
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
