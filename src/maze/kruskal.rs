use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::seq::SliceRandom;

use crate::{
    components::*,
    consts::*,
    grid::{Dir, Grid},
    states::GamePlayState,
};

#[derive(Component)]
pub struct MazeState {
    queue: Vec<(IVec2, Dir)>,
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

    commands.spawn(MazeState { queue });
}

pub fn generate(
    mut state_query: Query<&mut MazeState>,
    mut grid_query: Query<&mut Grid, With<Trees>>,
    mut next_state: ResMut<NextState<crate::GamePlayState>>,
) {
    let Ok(mut state) = state_query.get_single_mut() else {
        return;
    };

    let Ok(mut grid) = grid_query.get_single_mut() else {
        return;
    };

    for _ in 0..1 {
        loop {
            let Some((pos, dir)) = state.queue.pop() else {
                println!("Maze done");
                next_state.set(crate::GamePlayState::Playing);
                return;
            };

            if let Ok(_) = grid.remove_wall(pos, dir) {
                break;
            }
        }
    }
}
