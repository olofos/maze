use std::time::Duration;

use bevy::color::palettes::basic::SILVER;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;

#[cfg(not(target_arch = "wasm32"))]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

const GRID_WIDTH: usize = 32;
const GRID_HEIGHT: usize = 32;
const MARGIN: f32 = 16.0;
const PLAYFIELD_WIDTH: f32 = 1024.0;
const PLAYFIELD_HEIGHT: f32 = 1024.0;
const SCREEN_WIDTH: f32 = PLAYFIELD_WIDTH + MARGIN * 2.0;
const SCREEN_HEIGHT: f32 = PLAYFIELD_HEIGHT + MARGIN * 2.0;
const PIXEL_WIDTH: f32 = GRID_WIDTH as f32 / PLAYFIELD_WIDTH as f32;
const PIXEL_HEIGHT: f32 = GRID_HEIGHT as f32 / PLAYFIELD_HEIGHT as f32;
const PLAYER_WIDTH: f32 = 0.75;
const PLAYER_HEIGHT: f32 = 0.75;
const PLAYER_SPEED: f32 = GRID_WIDTH as f32 / 4.0;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Goal;

#[derive(Component)]
struct BackgroundSprite;

#[derive(Clone)]
struct Walls {
    up: Option<Entity>,
    down: Option<Entity>,
    left: Option<Entity>,
    right: Option<Entity>,
}

#[derive(Component)]
struct Grid {
    visited: Vec<i32>,
    walls: Vec<Walls>,
    sprites: Vec<Entity>,
}

#[derive(Component)]
struct Cursor {
    path: Vec<IVec2>,
    sprites: Vec<Entity>,
    default: IVec2,
    num: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Walls {
    fn get_mut(&mut self, dir: Dir) -> &mut Option<Entity> {
        match dir {
            Dir::Up => &mut self.up,
            Dir::Down => &mut self.down,
            Dir::Left => &mut self.left,
            Dir::Right => &mut self.right,
        }
    }
}

impl Dir {
    fn reverse(&self) -> Self {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
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
    .add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()))
    .add_systems(Startup, setup)
    .add_systems(Update, close_on_esc)
    .add_systems(FixedUpdate, move_player)
    .add_systems(
        Update,
        move_cursor.run_if(on_timer(Duration::from_millis(10))),
    )
    .add_systems(FixedUpdate, check_goal);
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
) {
    let player_transform = player_query.single();
    let goal_transform = goal_query.single();

    if collide(player_transform, goal_transform) {
        ev_appexit.send(AppExit::Success);
    }
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
                (*pos + IVec2::Y, Some(Dir::Up)),
                (*pos + IVec2::X, Some(Dir::Right)),
                (*pos - IVec2::Y, Some(Dir::Down)),
                (*pos - IVec2::X, Some(Dir::Left)),
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
                    Some(dir) => {
                        *grid.walls[(pos.y * GRID_WIDTH as i32 + pos.x) as usize]
                            .get_mut(dir.reverse()) = None;
                        grid.walls[(old_pos.y * GRID_WIDTH as i32 + old_pos.x) as usize]
                            .get_mut(dir)
                            .take()
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

    for num in 1..=1 {
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

    let mut walls: Vec<Walls> = vec![
        Walls {
            up: None,
            down: None,
            left: None,
            right: None
        };
        GRID_WIDTH * GRID_HEIGHT
    ];

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
                    Name::from(format!("Horizontal Wall {x} {y}")),
                ))
                .id();

            walls[(y * GRID_WIDTH + x) as usize].up = Some(id);
            walls[((y + 1) * GRID_WIDTH + x) as usize].down = Some(id);
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
                    Name::from(format!("Vertical Wall {x} {y}")),
                ))
                .id();

            walls[(y * GRID_WIDTH + x) as usize].right = Some(id);
            walls[(y * GRID_WIDTH + x + 1) as usize].left = Some(id);
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
            Name::from(format!("Outer Wall {name}")),
        ));
    }

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
