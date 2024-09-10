use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;
use bevy::time::common_conditions::on_timer;

use bevy::window::PresentMode;
#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;
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
        Material2dPlugin::<tilemap::TilemapMaterial>::default(),
    ))
    .insert_state(GameState::default())
    .add_systems(Update, tilemap::construct_materials)
    .add_systems(Update, tilemap::update_tilemaps)
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
    .add_systems(Update, expand_image_array)
    .add_systems(FixedUpdate, generate_bg.run_if(on_timer(Duration::from_millis(500))))
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

#[derive(Component)]
struct TilemapLoader {
    tileset: Handle<Image>,
    expand: bool,
    grid_size: (u32, u32),
    num_tiles: u32,
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
        TilemapLoader {
            tileset: asset_server.load("tileset4.png"),
            expand: true,
            grid_size: (GRID_WIDTH as u32, GRID_HEIGHT as u32),
            num_tiles: 17,
        },
        Transform::default().with_translation(Vec3::new(0.0, 0.0, 5.0)),
        Trees,
    ));

    commands.spawn((
        ImageArray {
            image: asset_server.load("bg.png"),
            num_tiles: 7,
        },
        Tilemap::new(32, 32),
        Transform::default().with_translation(Vec3::new(0.0, 0.0, -5.0)),
        Ground,
    ));
}

#[derive(Component)]
struct Trees;

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct ImageArray {
    image: Handle<Image>,
    num_tiles: u32,
}

fn expand_image_array(
    mut commands: Commands,
    query: Query<(Entity, &ImageArray)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, image_array) in query.iter() {
        let Some(image) = images.get_mut(&image_array.image) else {
            continue;
        };
        image.reinterpret_stacked_2d_as_array(image_array.num_tiles);
        let handle = image_array.image.clone();
        commands
            .entity(entity)
            .remove::<ImageArray>()
            .insert(handle);
    }
}

fn construct_tilemap(
    mut commands: Commands,
    query: Query<(Entity, &TilemapLoader), Without<Tilemap>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, loader) in query.iter() {
        println!("Load {entity}");
        let tileset = loader.tileset.clone();

        let Some(image) = images.remove(&tileset) else {
            continue;
        };

        let mut tileset_image = if loader.expand {
            tileset::expand(image)
        } else {
            image
        };

        if tileset_image.texture_descriptor.size.depth_or_array_layers == 1 {
            tileset_image.reinterpret_stacked_2d_as_array(loader.num_tiles);
        }

        commands.entity(entity).insert((
            images.add(tileset_image),
            Tilemap::new(loader.grid_size.0, loader.grid_size.1),
        ));
    }
}

fn generate_bg(mut commands: Commands, mut query: Query<(Entity, &mut Tilemap), With<Ground>>) {
    let Ok((entity, mut tilemap)) = query.get_single_mut() else {
        return;
    };

    let width = tilemap.width;
    let height = tilemap.height;

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
