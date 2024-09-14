use bevy::prelude::*;

use crate::{
    components::*,
    consts::*,
    grid::{Dir, Grid},
};

#[derive(Component)]
pub struct MazeCursor {
    path: Vec<IVec2>,
    default: IVec2,
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
    mut grid_query: Query<&mut Grid, With<Trees>>,
    mut next_state: ResMut<NextState<crate::GamePlayState>>,
) {
    let Ok(mut grid) = grid_query.get_single_mut() else {
        return;
    };

    for _ in 0..1 {
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
                        && !grid.is_visited(*p)
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
                if grid.is_visited(cursor.default) {
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
    grid_query: Query<&Grid>,
    mut cover_query: Query<&mut crate::tilemap::Tilemap, With<Cover>>,
) {
    let Ok(mut cover) = cover_query.get_single_mut() else {
        return;
    };

    let Ok(grid) = grid_query.get_single() else {
        return;
    };

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if !grid.is_visited(IVec2::new(x as i32, y as i32)) {
                cover.data[y * GRID_WIDTH + x] = 0;
            } else {
                cover.data[y * GRID_WIDTH + x] = (cover.data[y * GRID_WIDTH + x] + 1).clamp(0, 64);
            }
        }
    }
}

pub fn update_overlay(
    grid_query: Query<&Grid>,
    mut overlay_query: Query<&mut crate::overlay::Overlay>,
) {
    let Ok(mut overlay) = overlay_query.get_single_mut() else {
        return;
    };

    let Ok(grid) = grid_query.get_single() else {
        return;
    };

    overlay.data.clone_from(&grid.region);
}
