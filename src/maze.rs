use bevy::prelude::*;

use crate::{components::*, consts::*, tilemap};

#[derive(Component)]
pub struct MazeCursor {
    path: Vec<IVec2>,
    default: IVec2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    North = 0b0001,
    East = 0b0010,
    South = 0b0100,
    West = 0b1000,
}

struct GridRef<'a> {
    data: &'a mut Vec<u8>,
    // width: usize,
    // height: usize,
}

impl Dir {
    pub fn reverse(&self) -> Self {
        match self {
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::West => Dir::East,
            Dir::East => Dir::West,
        }
    }
}

impl From<Dir> for IVec2 {
    fn from(dir: Dir) -> Self {
        match dir {
            Dir::North => IVec2::Y,
            Dir::East => IVec2::X,
            Dir::South => -IVec2::Y,
            Dir::West => -IVec2::X,
        }
    }
}
impl<'a> GridRef<'a> {
    fn new(tilemap: &'a mut tilemap::Tilemap) -> Self {
        Self {
            data: &mut tilemap.data,
        }
    }

    fn is_visited(&self, pos: &IVec2) -> bool {
        self.data[(pos.y as usize) * GRID_WIDTH + pos.x as usize] > 0
    }

    fn remove_wall(&mut self, pos: IVec2, dir: Dir) {
        self.data[(pos.y as usize) * GRID_WIDTH + pos.x as usize] |= dir as u8;
        let pos: IVec2 = pos + IVec2::from(dir);
        self.data[(pos.y as usize) * GRID_WIDTH + pos.x as usize] |= dir.reverse() as u8;
    }
}

pub fn setup(mut commands: Commands) {
    for num in 1..=NUM_CURSORS {
        let x = ((num - 1) % 2) * (GRID_WIDTH as i32 - 1);
        let y = ((num - 1) / 2) * (GRID_HEIGHT as i32 - 1);
        commands.spawn((
            MazeCursor {
                path: vec![],
                default: IVec2::new(x, y),
            },
            Name::from(format!("Cursor {num}")),
        ));
    }
}

pub fn generate(
    mut cursor_query: Query<&mut MazeCursor>,
    mut tilemap_query: Query<&mut tilemap::Tilemap, With<Trees>>,
    mut next_state: ResMut<NextState<crate::GamePlayState>>,
) {
    let Ok(mut tilemap) = tilemap_query.get_single_mut() else {
        return;
    };
    let mut grid = GridRef::new(&mut tilemap);

    // for _ in 0..100 {
    loop {
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
                        && (old_pos == &cursor.default
                            || !(old_pos.x == GRID_WIDTH as i32 - 1
                                && old_pos.y == GRID_HEIGHT as i32 - 1))
                });

                if possibilities.is_empty() {
                    cursor.path.pop();
                    // commands.entity(cursor.sprites.pop().unwrap()).despawn();
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

            cursor.path.push(pos);

            if let Some(old_pos) = old_pos {
                if let Some(dir) = dir {
                    grid.remove_wall(old_pos, dir);
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

pub fn update_cover(
    tilemap_query: Query<&tilemap::Tilemap, (With<Trees>, Without<Cover>)>,
    mut cover_query: Query<&mut tilemap::Tilemap, With<Cover>>,
) {
    let Ok(mut cover) = cover_query.get_single_mut() else {
        return;
    };

    let Ok(tilemap) = tilemap_query.get_single() else {
        return;
    };

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if tilemap.data[y * GRID_WIDTH + x] == 0 {
                cover.data[y * GRID_WIDTH + x] = 0;
            } else {
                cover.data[y * GRID_WIDTH + x] = (cover.data[y * GRID_WIDTH + x] + 1).clamp(0, 64);
            }
        }
    }
}
