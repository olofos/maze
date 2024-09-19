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
    let mut tiles = Vec::new();

    let mut min_len = 4;
    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            let i = y * GRID_WIDTH + x;
            let m = grid.possible_moves(IVec2::new(x as i32, y as i32));

            if !state.fixed[i] && !m.is_empty() && m.len() <= min_len {
                min_len = min_len.min(m.len());
                tiles.push((i, m));
            }
        }
    }
    tiles.retain(|(_, v)| v.len() == min_len);

    if tiles.is_empty() {
        println!("Tile list is empty. Retrying...");
        state.fixed.fill(false);
        return;
    }

    let mut rng = rand::thread_rng();

    let tile_index = rng.gen_range(0..tiles.len());
    let (min_index, min_moves) = &tiles[tile_index];

    let min_y = min_index / GRID_WIDTH;
    let min_x = min_index % GRID_WIDTH;
    let pos = IVec2::new(min_x as i32, min_y as i32);

    for dir in min_moves {
        if rng.gen_bool(0.5) {
            let _ = grid.remove_wall(pos, *dir);
        }
    }

    state.fixed[*min_index] = true;
}
