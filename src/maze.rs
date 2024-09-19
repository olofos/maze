use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
    components::*,
    consts::*,
    grid::Grid,
    states::{AppState, GamePlayState},
};

mod backtracking;
mod kruskal;
mod wfc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum MazeType {
    #[default]
    Backtracking,
    Kruskal,
    Wfc,
}

#[derive(Component)]
pub enum MazeState {
    Backtracking(backtracking::MazeState),
    Kruskal(kruskal::MazeState),
    Wfc(wfc::MazeState),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Plugin {
    pub maze_type: MazeType,
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let maze_type = self.maze_type;
        app.add_systems(
            OnEnter(GamePlayState::GeneratingMaze),
            move |commands: Commands| setup(commands, maze_type),
        )
        .add_systems(
            Update,
            generate
                .run_if(on_timer(Duration::from_millis(MAZE_GEN_TIME_MS)))
                .run_if(in_state(GamePlayState::GeneratingMaze)),
        )
        .add_systems(
            Update,
            (update_cover, update_overlay).run_if(in_state(AppState::InGame)),
        );
    }
}

pub fn setup(mut commands: Commands, maze_type: MazeType) {
    let state = match maze_type {
        MazeType::Backtracking => MazeState::Backtracking(backtracking::init()),
        MazeType::Kruskal => MazeState::Kruskal(kruskal::init()),
        MazeType::Wfc => MazeState::Wfc(wfc::init()),
    };
    commands.spawn(state);
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

    let state = &mut *state;
    let grid = &mut *grid;

    for _ in 0..1 {
        if grid.regions.num_sets() == 1 {
            println!("Maze done");
            next_state.set(crate::GamePlayState::Playing);
            return;
        }

        match state {
            MazeState::Backtracking(maze_state) => backtracking::step(maze_state, grid),
            MazeState::Kruskal(maze_state) => kruskal::step(maze_state, grid),
            MazeState::Wfc(maze_state) => wfc::step(maze_state, grid),
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
    grid_query: Query<&Grid, Changed<Grid>>,
    mut overlay_query: Query<&mut crate::overlay::Overlay>,
) {
    let Ok(mut overlay) = overlay_query.get_single_mut() else {
        return;
    };

    let Ok(grid) = grid_query.get_single() else {
        return;
    };

    for (i, n) in grid.regions.values().enumerate() {
        overlay.data[i] = (n & 0xFF) as u8;
    }

    println!(
        "Num sets: {},  Max depth {}",
        grid.regions.num_sets(),
        (0..GRID_WIDTH * GRID_HEIGHT)
            .map(|i| grid.regions.depth(i))
            .max()
            .unwrap()
    );
}
