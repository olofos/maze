use bevy::prelude::*;
use rand::seq::SliceRandom;

use crate::{
    consts::*,
    grid::{Dir, Grid},
};

#[derive(Component)]
pub struct MazeState {
    queue: Vec<(IVec2, Dir)>,
}

pub fn init() -> MazeState {
    let mut queue = Vec::new();
    for y in 0..GRID_HEIGHT as i32 {
        for x in 0..GRID_WIDTH as i32 {
            if y < GRID_HEIGHT as i32 - 1 {
                queue.push((IVec2::new(x, y), Dir::North));
            }

            if x < GRID_WIDTH as i32 - 1 {
                queue.push((IVec2::new(x, y), Dir::East));
            }
        }
    }

    let mut rng = rand::thread_rng();
    queue.shuffle(&mut rng);

    MazeState { queue }
}

pub fn step(state: &mut MazeState, grid: &mut Grid) {
    loop {
        let (pos, dir) = state
            .queue
            .pop()
            .expect("The queue should not be empty yet");

        if grid.remove_wall(pos, dir).is_ok() {
            break;
        }
    }
}
