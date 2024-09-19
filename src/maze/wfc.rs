use bevy::math::IVec2;

use crate::consts::*;
use crate::grid::Grid;
use rand::Rng;

pub struct MazeState {
    fixed: Vec<bool>,
}

pub fn init() -> MazeState {
    MazeState {
        fixed: vec![false; GRID_WIDTH * GRID_HEIGHT],
    }
}

pub fn step(state: &mut MazeState, grid: &mut Grid) {
    let mut moves = Vec::new();
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            moves.push(grid.possible_moves(IVec2::new(x as i32, y as i32)));
        }
    }

    let mut rng = rand::thread_rng();

    let mut cells = moves
        .iter()
        .enumerate()
        .filter(|(i, v)| !state.fixed[*i] && !v.is_empty())
        .collect::<Vec<_>>();

    if cells.is_empty() {
        println!("Cells is empty. Retrying...");
        state.fixed.fill(false);
        return;
    }

    cells.sort_by_key(|(_, v)| v.len());
    let min_len = cells[0].1.len();
    cells.retain(|(_, v)| v.len() == min_len);
    let cells_index = rng.gen_range(0..cells.len());
    let (min_index, min_moves) = cells[cells_index];

    let min_y = min_index / GRID_WIDTH;
    let min_x = min_index % GRID_WIDTH;
    let pos = IVec2::new(min_x as i32, min_y as i32);

    for mov in min_moves {
        if rng.gen_bool(0.5) {
            let _ = grid.remove_wall(pos, *mov);
        }
    }

    state.fixed[min_index] = true;
}
