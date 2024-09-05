use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
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
    GeneratingMaze,
    Playing,
    LevelDone,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Maze".to_string(),
            resizable: false,
            resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
            ..default()
        }),
        ..default()
    }))
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
    .add_systems(FixedUpdate, check_goal.run_if(in_state(GameState::Playing)));
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

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(GRID_WIDTH as f32 / 2., GRID_HEIGHT as f32 / 2., 0.0),
            scale: Vec3::new(PIXEL_WIDTH, PIXEL_HEIGHT, 1.0),
            ..default()
        },
        ..default()
    });
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
