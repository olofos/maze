use crate::{consts::*, grid::Grid};
use bevy::prelude::*;

struct MazeCursor {
    path: Vec<IVec2>,
}

#[derive(Component)]
pub struct MazeState {
    cursors: Vec<MazeCursor>,
}

pub fn init() -> MazeState {
    let corner = [
        IVec2::new(0, 0),
        IVec2::new(GRID_WIDTH as i32 - 1, GRID_HEIGHT as i32 - 1),
        IVec2::new(GRID_WIDTH as i32 - 1, 0),
        IVec2::new(0, GRID_HEIGHT as i32 - 1),
    ];
    let cursors = (0..NUM_CURSORS as usize)
        .map(|n| MazeCursor {
            path: vec![corner[n]],
        })
        .collect();
    MazeState { cursors }
}

pub fn step(state: &mut MazeState, grid: &mut Grid) {
    for cursor in &mut state.cursors {
        let Some(pos) = cursor.path.last().copied() else {
            continue;
        };

        let possibilities = grid.possible_moves(pos);

        if possibilities.is_empty() {
            cursor.path.pop();
            continue;
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..possibilities.len());
        let dir = possibilities[index];

        let _ = grid.remove_wall(pos, dir);
        cursor.path.push(pos + IVec2::from(dir));
    }
}
