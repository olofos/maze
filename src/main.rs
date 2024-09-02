use std::time::Duration;

use bevy::{
    math::{
        bounding::{Aabb2d, IntersectsVolume},
        ivec2,
    },
    prelude::*,
    time::common_conditions::on_timer,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy::color::palettes::basic::SILVER;

const GRID_WIDTH: usize = 8;
const GRID_HEIGHT: usize = 8;
const MARGIN: f32 = 16.0;
const PLAYFIELD_WIDTH: f32 = 1024.0;
const PLAYFIELD_HEIGHT: f32 = 1024.0;
const SCREEN_WIDTH: f32 = PLAYFIELD_WIDTH + MARGIN * 2.0;
const SCREEN_HEIGHT: f32 = PLAYFIELD_HEIGHT + MARGIN * 2.0;
const CELL_WIDTH: f32 = PLAYFIELD_WIDTH / GRID_WIDTH as f32;
const CELL_HEIGHT: f32 = PLAYFIELD_HEIGHT / GRID_HEIGHT as f32;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Grid {
    visited: Vec<bool>,
    walls: Vec<[Option<Entity>; 4]>,
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
    fn new() -> Self {
        Self {
            visited: vec![false; GRID_WIDTH * GRID_HEIGHT],
            walls: vec![[None; 4]; GRID_WIDTH * GRID_HEIGHT],
        }
    }

    fn is_visited(&self, pos: &IVec2) -> bool {
        self.visited[(pos.y as usize) * GRID_WIDTH + pos.x as usize]
    }

    fn visit(&mut self, pos: &IVec2) {
        self.visited[(pos.y as usize) * GRID_WIDTH + pos.x as usize] = true;
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
        .add_systems(FixedUpdate, mover_player)
        .add_plugins(WorldInspectorPlugin::new().run_if(
            bevy::input::common_conditions::input_toggle_active(false, KeyCode::Backquote),
        ))
        .add_systems(
            Update,
            move_cursor.run_if(on_timer(Duration::from_millis(10))),
        )
        .run();
}

fn move_cursor(mut commands: Commands, mut cursor: Query<&mut Cursor>, mut grid: Query<&mut Grid>) {
    let mut cursor = cursor.single_mut();
    let mut grid = grid.single_mut();

    let old_pos = cursor.path.last().copied();

    let (pos, dir) = if let Some(pos) = cursor.path.last() {
        let mut possibilities = vec![
            (*pos + IVec2::Y, Some(0)),
            (*pos + IVec2::X, Some(1)),
            (*pos - IVec2::Y, Some(2)),
            (*pos - IVec2::X, Some(3)),
        ];

        possibilities.retain(|(pos, _)| {
            pos.x >= 0
                && pos.x < GRID_WIDTH as i32
                && pos.y >= 0
                && pos.y < GRID_HEIGHT as i32
                && !grid.is_visited(pos)
        });
        if pos.x == GRID_WIDTH as i32 - 1 && pos.y == GRID_HEIGHT as i32 - 1 {
            possibilities.clear();
        }

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
        if grid.is_visited(&IVec2::ZERO) {
            return;
        }
        (ivec2(0, 0), None)
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
    grid.visit(&pos);

    if let Some(id) = {
        if let Some(old_pos) = old_pos {
            match dir {
                Some(n) => {
                    grid.walls[(pos.y * GRID_WIDTH as i32 + pos.x) as usize][(n + 2) % 4] = None;
                    grid.walls[(old_pos.y * GRID_WIDTH as i32 + old_pos.x) as usize][n].take()
                }
                _ => None,
            }
        } else {
            None
        }
    } {
        if let Some(mut e) = commands.get_entity(id) {
            e.despawn();
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: Vec3::new(PLAYFIELD_WIDTH / 2., PLAYFIELD_HEIGHT / 2., 0.0),
            ..default()
        },
        ..default()
    });

    commands.spawn((Cursor::default(), Name::from("Cursor")));

    let mut walls: Vec<[Option<Entity>; 4]> = vec![[None; 4]; GRID_WIDTH * GRID_HEIGHT];

    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT - 1 {
            let id = commands
                .spawn((
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
                    Collider,
                    Name::from(format!("Horizontal Wall {x} {y}")),
                ))
                .id();

            walls[(y * GRID_WIDTH + x) as usize][0] = Some(id);
            walls[((y + 1) * GRID_WIDTH + x) as usize][2] = Some(id);
        }
    }
    for x in 0..GRID_WIDTH - 1 {
        for y in 0..GRID_HEIGHT {
            let id = commands
                .spawn((
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
                    Collider,
                    Name::from(format!("Vertical Wall {x} {y}")),
                ))
                .id();

            walls[(y * GRID_WIDTH + x) as usize][1] = Some(id);
            walls[(y * GRID_WIDTH + x + 1) as usize][3] = Some(id);
        }
    }

    commands.spawn(Grid {
        visited: vec![false; GRID_WIDTH * GRID_HEIGHT],
        walls,
    });

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
        Collider,
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
        Collider,
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
        Collider,
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
        Collider,
        Name::from("Outer Wall Top"),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(CELL_WIDTH / 2.0, CELL_HEIGHT / 2.0, 2.),
                scale: Vec3::new(CELL_WIDTH / 2.0, CELL_HEIGHT / 2.0, 1.0),
                ..default()
            },
            ..default()
        },
        Name::from("Player"),
        Player,
    ));
}

fn mover_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Transform, With<Player>>,
    collider_query: Query<&Transform, (With<Collider>, Without<Player>)>,
    time: Res<Time>,
) {
    const SPEED: f32 = 256.0;
    let mut player_transform = player_query.single_mut();

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

    direction *= SPEED * time.delta_seconds();

    let new_translation = player_transform.translation + direction;
    let player_aabb = Aabb2d::new(
        new_translation.truncate(),
        player_transform.scale.truncate() / 2.,
    );

    for collider_transform in collider_query.iter() {
        let aabb = Aabb2d::new(
            collider_transform.translation.truncate(),
            collider_transform.scale.truncate() / 2.,
        );

        if aabb.min.x > player_aabb.max.x
            || aabb.max.x < player_aabb.min.x
            || aabb.min.y > player_aabb.max.y
            || aabb.max.y < player_aabb.min.y
        {
            continue;
        }

        let collide_left = aabb.min.x < player_aabb.min.x;
        let collide_right = aabb.min.x < player_aabb.max.x;
        let collide_up = aabb.min.y < player_aabb.max.y;
        let collide_down = aabb.min.y < player_aabb.min.y;

        if (collide_right && direction.x > 0.0) || (collide_left && direction.x < 0.0) {
            direction.x = 0.0;
        }

        if (collide_up && direction.y > 0.0) || (collide_down && direction.y < 0.0) {
            direction.y = 0.0;
        }
    }

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
