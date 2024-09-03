use std::time::Duration;

use bevy::{math::bounding::Aabb2d, prelude::*, time::common_conditions::on_timer};

#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use bevy::color::palettes::basic::SILVER;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

const GRID_WIDTH: usize = 32;
const GRID_HEIGHT: usize = 32;
const MARGIN: f32 = 16.0;
const PLAYFIELD_WIDTH: f32 = 1024.0;
const PLAYFIELD_HEIGHT: f32 = 1024.0;
const SCREEN_WIDTH: f32 = PLAYFIELD_WIDTH + MARGIN * 2.0;
const SCREEN_HEIGHT: f32 = PLAYFIELD_HEIGHT + MARGIN * 2.0;
const PIXEL_WIDTH: f32 = GRID_WIDTH as f32 / PLAYFIELD_WIDTH as f32;
const PIXEL_HEIGHT: f32 = GRID_HEIGHT as f32 / PLAYFIELD_HEIGHT as f32;
const PLAYER_SPEED: f32 = 4.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct BackgroundSprite;

#[derive(Component)]
struct Grid {
    visited: Vec<i32>,
    walls: Vec<[Option<Entity>; 4]>,
    sprites: Vec<Entity>,
}

#[derive(Component)]
struct Cursor {
    path: Vec<IVec2>,
    sprites: Vec<Entity>,
    default: IVec2,
    num: i32,
}

impl Grid {
    fn is_visited(&self, pos: &IVec2) -> bool {
        self.visited[(pos.y as usize) * GRID_WIDTH + pos.x as usize] > 0
    }

    fn visit(&mut self, pos: &IVec2, num: i32) {
        self.visited[(pos.y as usize) * GRID_WIDTH + pos.x as usize] = num;
    }
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
    .add_systems(Startup, setup)
    .add_systems(Update, close_on_esc)
    .add_systems(FixedUpdate, mover_player)
    .add_systems(
        Update,
        move_cursor.run_if(on_timer(Duration::from_millis(50))),
    );
    #[cfg(not(target_arch = "wasm32"))]
    app.add_plugins(WorldInspectorPlugin::new().run_if(
        bevy::input::common_conditions::input_toggle_active(false, KeyCode::Backquote),
    ));
    app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));

    app.run();
}

fn move_cursor(
    mut commands: Commands,
    mut cursor_query: Query<&mut Cursor>,
    mut grid_query: Query<&mut Grid>,
    mut background_query: Query<&mut Sprite, With<BackgroundSprite>>,
) {
    for mut cursor in cursor_query.iter_mut() {
        let mut grid = grid_query.single_mut();

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

            if possibilities.is_empty() {
                cursor.path.pop();
                commands.entity(cursor.sprites.pop().unwrap()).despawn();
                continue;
            } else {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..possibilities.len());
                possibilities[index]
            }
        } else {
            if grid.is_visited(&cursor.default) {
                continue;
            }
            (cursor.default, None)
        };

        let (cx, cy, w, h) = if let Some(prev_pos) = old_pos {
            if prev_pos.x == pos.x {
                (
                    pos.x as f32,
                    (prev_pos.y + pos.y) as f32 / 2.0,
                    0.25,
                    0.75 + (prev_pos.y - pos.y).abs() as f32 / 2.0,
                )
            } else if prev_pos.y == pos.y {
                (
                    (prev_pos.x + pos.x) as f32 / 2.0,
                    pos.y as f32,
                    0.75 + (prev_pos.x - pos.x).abs() as f32 / 2.0,
                    0.25,
                )
            } else {
                (pos.x as f32, pos.y as f32, 0.25, 0.25)
            }
        } else {
            (pos.x as f32, pos.y as f32, 0.25, 0.25)
        };

        let sprite_id = commands
            .spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::from(SILVER),
                        ..default()
                    },
                    transform: Transform {
                        translation: Vec3::new(cx + 0.5, cy + 0.5, 1.),
                        scale: Vec3::new(w, h, 1.0),
                        ..default()
                    },
                    ..default()
                },
                Name::from(format!("Cursor {}", pos)),
            ))
            .id();

        cursor.path.push(pos);
        cursor.sprites.push(sprite_id);
        grid.visit(&pos, cursor.num);

        let colors = [
            Color::srgb(0.2, 0.0, 0.0),
            Color::srgb(0.0, 0.2, 0.0),
            Color::srgb(0.2, 0.0, 0.2),
            Color::srgb(0.0, 0.0, 0.2),
        ];

        if let Ok(mut sprite) =
            background_query.get_mut(grid.sprites[(pos.y * GRID_WIDTH as i32 + pos.x) as usize])
        {
            sprite.color = colors[(cursor.num - 1) as usize];
        }

        if let Some(id) = {
            if let Some(old_pos) = old_pos {
                match dir {
                    Some(n) => {
                        grid.walls[(pos.y * GRID_WIDTH as i32 + pos.x) as usize][(n + 2) % 4] =
                            None;
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

    for num in 1..=4 {
        let x = ((num - 1) % 2) * (GRID_WIDTH as i32 - 1);
        let y = ((num - 1) / 2) * (GRID_HEIGHT as i32 - 1);
        commands.spawn((
            Cursor {
                path: vec![],
                sprites: vec![],
                default: IVec2::new(x, y),
                num,
            },
            Name::from(format!("Cursor {num}")),
        ));
    }

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
                            translation: Vec3::new(x as f32 + 0.5, y as f32 + 1.0, 1.),
                            scale: Vec3::new(1.0, PIXEL_HEIGHT, 1.0),
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
                            translation: Vec3::new(x as f32 + 1.0, y as f32 + 0.5, 1.),
                            scale: Vec3::new(PIXEL_WIDTH, 1.0, 1.0),
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

    let mut sprites = vec![];
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let id = commands
                .spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgb(0.0, 0.0, 0.0),
                            ..default()
                        },
                        transform: Transform {
                            translation: Vec3::new(x as f32 + 0.5, y as f32 + 0.5, -1.),
                            scale: Vec3::new(1.0, 1.0, 1.0),
                            ..default()
                        },
                        ..default()
                    },
                    BackgroundSprite,
                    Name::from(format!("Background {x} {y}")),
                ))
                .id();
            sprites.push(id);
        }
    }

    commands.spawn(Grid {
        visited: vec![0; GRID_WIDTH * GRID_HEIGHT],
        walls,
        sprites,
    });

    for (x, y, w, h, name) in [
        (
            0.0,
            0.5,
            3.0 * PIXEL_WIDTH,
            GRID_HEIGHT as f32 + 2.0 * PIXEL_HEIGHT,
            "Left",
        ),
        (
            1.0,
            0.5,
            3.0 * PIXEL_WIDTH,
            GRID_HEIGHT as f32 + 2.0 * PIXEL_HEIGHT,
            "Right",
        ),
        (
            0.5,
            1.0,
            GRID_WIDTH as f32 + 2.0 * PIXEL_WIDTH,
            3.0 * PIXEL_HEIGHT,
            "Top",
        ),
        (
            0.5,
            0.0,
            GRID_WIDTH as f32 + 2.0 * PIXEL_WIDTH,
            3.0 * PIXEL_HEIGHT,
            "Bottom",
        ),
    ] {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::from(SILVER),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(x * GRID_WIDTH as f32, y * GRID_HEIGHT as f32, 1.),
                    scale: Vec3::new(w, h, 1.0),
                    ..default()
                },
                ..default()
            },
            Collider,
            Name::from(format!("Outer Wall {name}")),
        ));
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(1.0, 0.0, 0.0),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(0.5, 0.5, 2.),
                scale: Vec3::new(0.5, 0.5, 1.0),
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

    direction *= PLAYER_SPEED * time.delta_seconds();

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
