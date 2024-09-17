use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};

use crate::{
    components::*,
    consts::*,
    grid::{Dir, Grid},
    states::GamePlayState,
};

#[derive(Component)]
pub struct MazeCursor {
    path: Vec<IVec2>,
    default: IVec2,
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
    for num in 0..NUM_CURSORS as usize {
        commands.spawn((
            MazeCursor {
                path: vec![],
                default: corner[num],
            },
            Name::from(format!("Cursor {}", num + 1)),
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
