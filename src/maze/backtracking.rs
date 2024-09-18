use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{components::*, consts::*, grid::Grid, states::GamePlayState};

struct MazeCursor {
    path: Vec<IVec2>,
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
            path: vec![corner[n]],
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

        if grid.regions.num_sets() == 1 {
            println!("Maze done");
            next_state.set(crate::GamePlayState::Playing);
            return;
        }
    }
}
