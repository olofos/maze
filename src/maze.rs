use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
};

use crate::{components::*, consts::*, TilemapMaterial};

#[derive(Component)]
pub struct MazeCursor {
    path: Vec<IVec2>,
    sprites: Vec<Entity>,
    default: IVec2,
    num: i32,
}

pub fn setup(mut commands: Commands) {
    for num in 1..=NUM_CURSORS {
        let x = ((num - 1) % 2) * (GRID_WIDTH as i32 - 1);
        let y = ((num - 1) / 2) * (GRID_HEIGHT as i32 - 1);
        commands.spawn((
            MazeCursor {
                path: vec![],
                sprites: vec![],
                default: IVec2::new(x, y),
                num,
            },
            Name::from(format!("Cursor {num}")),
        ));
    }

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
                            color: Color::srgb(0.75, 0.75, 0.75),
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
                            color: Color::srgb(0.75, 0.75, 0.75),
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

    commands.spawn(Grid {
        visited: vec![0; GRID_WIDTH * GRID_HEIGHT],
        walls,
    });

    for (x, y, name) in [
        (0.0, 0.5, "Left"),
        (1.0, 0.5, "Right"),
        (0.5, 1.0, "Top"),
        (0.5, 0.0, "Bottom"),
    ] {
        let scale = if y == 0.5 {
            Vec3::new(
                3.0 * PIXEL_WIDTH,
                GRID_HEIGHT as f32 + 2.0 * PIXEL_HEIGHT,
                1.0,
            )
        } else {
            Vec3::new(
                GRID_WIDTH as f32 + 2.0 * PIXEL_WIDTH,
                3.0 * PIXEL_HEIGHT,
                1.0,
            )
        };
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.75, 0.75, 0.75),
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(x * GRID_WIDTH as f32, y * GRID_HEIGHT as f32, 1.),
                    scale,
                    ..default()
                },
                ..default()
            },
            Name::from(format!("Outer Wall {name}")),
        ));
    }
}

pub fn generate(
    mut commands: Commands,
    mut cursor_query: Query<&mut MazeCursor>,
    mut grid_query: Query<&mut Grid>,
    mut next_state: ResMut<NextState<crate::GameState>>,
    mut images: ResMut<Assets<Image>>,
    tilemap_query: Query<&crate::TilemapHandle>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
) {
    let mut num_completed = 0;
    let mut grid = grid_query.single_mut();
    for mut cursor in cursor_query.iter_mut() {
        let old_pos = cursor.path.last().copied();

        let (pos, dir) = if let Some(old_pos) = &old_pos {
            let mut possibilities = vec![
                (*old_pos + IVec2::Y, Some(Dir::Up)),
                (*old_pos + IVec2::X, Some(Dir::Right)),
                (*old_pos - IVec2::Y, Some(Dir::Down)),
                (*old_pos - IVec2::X, Some(Dir::Left)),
            ];

            possibilities.retain(|(p, _)| {
                p.x >= 0
                    && p.x < GRID_WIDTH as i32
                    && p.y >= 0
                    && p.y < GRID_HEIGHT as i32
                    && !grid.is_visited(p)
                    && !((old_pos != &cursor.default)
                        && (old_pos.x == 0 || old_pos.x == GRID_WIDTH as i32 - 1)
                        && (old_pos.y == 0 || old_pos.y == GRID_HEIGHT as i32 - 1))
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
                num_completed += 1;
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
                        color: Color::srgb(0.75, 0.75, 0.75),
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

    if let Ok(tilemap) = tilemap_query.get_single() {
        let mat = materials.get_mut(&tilemap.material).unwrap();
        let mut data = Vec::with_capacity(GRID_WIDTH * GRID_HEIGHT);
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let mut val = 0b1111;
                if grid.walls[y * GRID_WIDTH + x].up.is_some() || y == GRID_HEIGHT - 1 {
                    val &= !0b0001;
                }
                if grid.walls[y * GRID_WIDTH + x].right.is_some() || x == GRID_WIDTH - 1 {
                    val &= !0b0010;
                }
                if grid.walls[y * GRID_WIDTH + x].down.is_some() || y == 0 {
                    val &= !0b0100;
                }
                if grid.walls[y * GRID_WIDTH + x].left.is_some() || x == 0 {
                    val &= !0b1000;
                }

                data.push(val);
            }
        }

        let mut tilemap_image = Image::new(
            Extent3d {
                width: GRID_WIDTH as u32,
                height: GRID_HEIGHT as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::R8Uint,
            RenderAssetUsages::all(),
        );
        tilemap_image.sampler = ImageSampler::nearest();

        let tilemap_handle = images.add(tilemap_image);
        mat.tilemap_texture = Some(tilemap_handle);
    }

    if num_completed == NUM_CURSORS {
        println!("Maze done");
        next_state.set(crate::GameState::Playing);
    }
}
