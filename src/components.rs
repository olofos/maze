use bevy::prelude::*;

use crate::consts::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Goal;

#[derive(Debug, Clone)]
pub struct Walls {
    pub up: Option<Entity>,
    pub down: Option<Entity>,
    pub left: Option<Entity>,
    pub right: Option<Entity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component)]
pub struct Grid {
    pub visited: Vec<i32>,
    pub walls: Vec<Walls>,
}

impl Walls {
    pub fn get_mut(&mut self, dir: Dir) -> &mut Option<Entity> {
        match dir {
            Dir::Up => &mut self.up,
            Dir::Down => &mut self.down,
            Dir::Left => &mut self.left,
            Dir::Right => &mut self.right,
        }
    }
}

impl Dir {
    pub fn reverse(&self) -> Self {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }
}

impl Grid {
    pub fn is_visited(&self, pos: &IVec2) -> bool {
        self.visited[(pos.y as usize) * GRID_WIDTH + pos.x as usize] > 0
    }

    pub fn visit(&mut self, pos: &IVec2, num: i32) {
        self.visited[(pos.y as usize) * GRID_WIDTH + pos.x as usize] = num;
    }
}
