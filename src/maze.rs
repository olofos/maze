use bevy::prelude::*;

use crate::{components::*, consts::*, tilemap};

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

    let walls: Vec<Walls> = vec![Walls::default(); GRID_WIDTH * GRID_HEIGHT];

    commands.spawn((
        Grid {
            visited: vec![0; GRID_WIDTH * GRID_HEIGHT],
            walls,
        },
        Name::from("Grid"),
    ));
}

pub fn generate(
    mut commands: Commands,
    mut cursor_query: Query<&mut MazeCursor>,
    mut grid_query: Query<&mut Grid>,
    mut next_state: ResMut<NextState<crate::GamePlayState>>,
) {
    let mut grid = grid_query.single_mut();

    for _ in 0..4 {
        // loop {
        let mut num_completed = 0;
        for mut cursor in cursor_query.iter_mut() {
            let old_pos = cursor.path.last().copied();

            let (pos, dir) = if let Some(old_pos) = &old_pos {
                let mut possibilities = vec![
                    (*old_pos + IVec2::Y, Some(Dir::North)),
                    (*old_pos + IVec2::X, Some(Dir::East)),
                    (*old_pos - IVec2::Y, Some(Dir::South)),
                    (*old_pos - IVec2::X, Some(Dir::West)),
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

            if let Some(old_pos) = old_pos {
                if let Some(dir) = dir {
                    *grid.walls[(pos.y * GRID_WIDTH as i32 + pos.x) as usize]
                        .get_mut(dir.reverse()) = false;
                    *grid.walls[(old_pos.y * GRID_WIDTH as i32 + old_pos.x) as usize]
                        .get_mut(dir) = false;
                }
            }
        }

        if num_completed == NUM_CURSORS {
            println!("Maze done");
            next_state.set(crate::GamePlayState::Playing);
            return;
        }
    }
}

pub fn update_tilemap(
    grid_query: Query<&Grid, Changed<Grid>>,
    mut tilemap_query: Query<&mut tilemap::Tilemap, With<crate::Trees>>,
) {
    let Ok(mut tilemap) = tilemap_query.get_single_mut() else {
        return;
    };

    let Ok(grid) = grid_query.get_single() else {
        return;
    };

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let mut val = 0b1111;
            if grid.walls[y * GRID_WIDTH + x].n || y == GRID_HEIGHT - 1 {
                val &= !0b0001;
            }
            if grid.walls[y * GRID_WIDTH + x].e || x == GRID_WIDTH - 1 {
                val &= !0b0010;
            }
            if grid.walls[y * GRID_WIDTH + x].s || y == 0 {
                val &= !0b0100;
            }
            if grid.walls[y * GRID_WIDTH + x].w || x == 0 {
                val &= !0b1000;
            }

            tilemap.data[y * GRID_WIDTH + x] = val;
        }
    }
}

pub fn update_cover(
    grid_query: Query<&Grid>,
    mut tilemap_query: Query<&mut tilemap::Tilemap, With<crate::Cover>>,
) {
    let Ok(mut tilemap) = tilemap_query.get_single_mut() else {
        return;
    };

    let Ok(grid) = grid_query.get_single() else {
        return;
    };

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.visited[y * GRID_WIDTH + x] == 0 {
                tilemap.data[y * GRID_WIDTH + x] = 0;
            } else {
                tilemap.data[y * GRID_WIDTH + x] =
                    (tilemap.data[y * GRID_WIDTH + x] + 1).clamp(0, 64);
            }
        }
    }
}
