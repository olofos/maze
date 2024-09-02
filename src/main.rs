use std::time::Duration;

use bevy::{math::ivec2, prelude::*, time::common_conditions::on_timer};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy::color::palettes::basic::SILVER;

const GRID_WIDTH: usize = 8;
const GRID_HEIGHT: usize = 8;
const MARGIN: f32 = 16.0;
const PLAYFIELD_WIDTH: f32 = 512.0;
const PLAYFIELD_HEIGHT: f32 = 512.0;
const SCREEN_WIDTH: f32 = PLAYFIELD_WIDTH + MARGIN * 2.0;
const SCREEN_HEIGHT: f32 = PLAYFIELD_HEIGHT + MARGIN * 2.0;
const CELL_WIDTH: f32 = PLAYFIELD_WIDTH / GRID_WIDTH as f32;
const CELL_HEIGHT: f32 = PLAYFIELD_HEIGHT / GRID_HEIGHT as f32;

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct Grid {
    pub visited: Vec<Vec<bool>>,
}

#[derive(Component)]
struct Cursor {
    path: Vec<IVec2>,
    sprites: Vec<Entity>,
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            path: Vec::new(),
            sprites: Vec::new(),
        }
    }
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            visited: vec![vec![false; width]; height],
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Maze".to_string(),
                resizable: false,
                resolution: (SCREEN_WIDTH, SCREEN_HEIGHT).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        // .add_systems(FixedUpdate, mover_player)
        .add_plugins(WorldInspectorPlugin::new().run_if(
            bevy::input::common_conditions::input_toggle_active(false, KeyCode::Backquote),
        ))
        .add_systems(
            Update,
            move_cursor.run_if(on_timer(Duration::from_millis(10))),
        )
        .run();
}

fn move_cursor(mut commands: Commands, mut query: Query<&mut Cursor>, mut grid: ResMut<Grid>) {
    let mut cursor = query.single_mut();

    let pos = if let Some(pos) = cursor.path.last() {
        let mut possibilities = vec![
            *pos + IVec2::Y,
            *pos - IVec2::Y,
            *pos + IVec2::X,
            *pos - IVec2::X,
        ];
        possibilities.retain(|&IVec2 { x, y }| {
            x >= 0
                && x < GRID_WIDTH as i32
                && y >= 0
                && y < GRID_HEIGHT as i32
                && !grid.visited[y as usize][x as usize]
        });

        if possibilities.is_empty() {
            cursor.path.pop();
            commands.entity(cursor.sprites.pop().unwrap()).despawn();
            return;
        } else {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..possibilities.len());
            possibilities[index]
        }
    } else {
        if grid.visited[0][0] {
            return;
        }
        ivec2(0, 0)
    };

    let sprite_id = commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::from(SILVER),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(
                        (pos.x as f32 + 0.5) * CELL_WIDTH,
                        (pos.y as f32 + 0.5) * CELL_HEIGHT,
                        1.,
                    ),
                    scale: Vec3::new(CELL_WIDTH / 2.0, CELL_HEIGHT / 2.0, 1.0),
                    ..default()
                },
                ..default()
            },
            Name::from(format!("Cursor {}", pos)),
        ))
        .id();

    cursor.path.push(pos);
    cursor.sprites.push(sprite_id);
    grid.visited[pos.y as usize][pos.x as usize] = true;
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(PLAYFIELD_WIDTH / 2., PLAYFIELD_HEIGHT / 2., 0.0),
            ..default()
        },
        ..default()
    });
    commands.insert_resource(Grid::new(GRID_WIDTH, GRID_HEIGHT));
    commands.spawn((Cursor::default(), Name::from("Cursor")));
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT - 1 {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::from(SILVER),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(
                            (x as f32 + 0.5) * CELL_WIDTH,
                            (y as f32 + 1.0) * CELL_HEIGHT,
                            1.,
                        ),
                        scale: Vec3::new(CELL_WIDTH, 1.0, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Name::from(format!("Horizontal Wall {x} {y}")),
            ));
        }
    }
    for x in 0..GRID_WIDTH - 1 {
        for y in 0..GRID_HEIGHT {
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::from(SILVER),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(
                            (x as f32 + 1.0) * CELL_WIDTH,
                            (y as f32 + 0.5) * CELL_HEIGHT,
                            1.,
                        ),
                        scale: Vec3::new(1.0, CELL_HEIGHT, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Name::from(format!("Vertical Wall {x} {y}")),
            ));
        }
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0., PLAYFIELD_HEIGHT / 2.0, 1.),
                scale: Vec3::new(1.0, PLAYFIELD_HEIGHT, 1.0),
                ..default()
            },
            ..default()
        },
        Name::from("Outer Wall Left"),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(PLAYFIELD_WIDTH, PLAYFIELD_HEIGHT / 2.0, 1.),
                scale: Vec3::new(1.0, PLAYFIELD_HEIGHT, 1.0),
                ..default()
            },
            ..default()
        },
        Name::from("Outer Wall Right"),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(PLAYFIELD_WIDTH / 2.0, 0., 1.),
                scale: Vec3::new(PLAYFIELD_WIDTH, 1.0, 1.0),
                ..default()
            },
            ..default()
        },
        Name::from("Outer Wall Bottom"),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(PLAYFIELD_WIDTH / 2.0, PLAYFIELD_HEIGHT, 1.),
                scale: Vec3::new(PLAYFIELD_WIDTH, 1.0, 1.0),
                ..default()
            },
            ..default()
        },
        Name::from("Outer Wall Top"),
    ));
}

fn mover_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    const SPEED: f32 = 128.0;
    let mut player_transform = player_query.single_mut();

    let mut direction = Vec3::new(0., 0., 0.);

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    direction = direction.normalize_or_zero() * SPEED * time.delta_seconds();

    player_transform.translation += direction;
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
