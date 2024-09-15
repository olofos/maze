use bevy::prelude::*;

use crate::{components::*, consts::*, grid::Grid, states::AppState};

mod backtracking;
mod kruskal;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MazeType {
    #[default]
    Backtracking,
    Kruskal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Plugin {
    pub maze_type: MazeType,
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        match self.maze_type {
            MazeType::Backtracking => backtracking::plugin(app),
            MazeType::Kruskal => kruskal::plugin(app),
        }
        app.add_systems(
            Update,
            (update_cover, update_overlay).run_if(in_state(AppState::InGame)),
        );
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
