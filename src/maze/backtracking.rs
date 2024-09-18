use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
    components::*,
    consts::*,
    grid::{Dir, Grid},
    states::GamePlayState,
};

struct MazeCursor {
    path: Vec<IVec2>,
    default: IVec2,
}

#[derive(Component)]
pub struct MazeState {
    cursors: Vec<MazeCursor>,
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GamePlayState::GeneratingMaze), setup)
        .add_systems(
            Update,
            generate
                .run_if(on_timer(Duration::from_millis(MAZE_GEN_TIME_MS)))
                .run_if(in_state(GamePlayState::GeneratingMaze)),
        );
}

pub fn setup(mut commands: Commands) {
    let corner = [
        IVec2::new(0, 0),
        IVec2::new(GRID_WIDTH as i32 - 1, GRID_HEIGHT as i32 - 1),
        IVec2::new(GRID_WIDTH as i32 - 1, 0),
        IVec2::new(0, GRID_HEIGHT as i32 - 1),
    ];
    let cursors = (0..NUM_CURSORS as usize)
        .map(|n| MazeCursor {
            path: vec![],
            default: corner[n],
        })
        .collect();
    commands.spawn(MazeState { cursors });
}

pub fn generate(
    mut state_query: Query<&mut MazeState>,
    mut grid_query: Query<&mut Grid, With<Trees>>,
    mut next_state: ResMut<NextState<crate::GamePlayState>>,
) {
    let Ok(mut grid) = grid_query.get_single_mut() else {
        return;
    };

    let Ok(mut state) = state_query.get_single_mut() else {
        return;
    };

    for _ in 0..1 {
        let mut num_completed = 0;
        // loop {
        for cursor in &mut state.cursors {
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
                        && grid.region(*p) != grid.region(*old_pos)
                        && (old_pos == &cursor.default
                            || !(old_pos.x == GRID_WIDTH as i32 - 1
                                && old_pos.y == GRID_HEIGHT as i32 - 1))
                });

                if possibilities.is_empty() {
                    cursor.path.pop();
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
                    let _ = grid.remove_wall(old_pos, dir);
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
