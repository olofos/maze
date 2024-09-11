use bevy::prelude::*;

use crate::consts::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Goal;

#[derive(Debug, Clone, Reflect)]
pub struct Walls {
    pub n: bool,
    pub s: bool,
    pub e: bool,
    pub w: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Component, Reflect)]
pub struct Grid {
    pub visited: Vec<i32>,
    pub walls: Vec<Walls>,
}

impl Default for Walls {
    fn default() -> Self {
        Self {
            n: true,
            s: true,
            e: true,
            w: true,
        }
    }
}

impl Walls {
    pub fn get_mut(&mut self, dir: Dir) -> &mut bool {
        match dir {
            Dir::Up => &mut self.n,
            Dir::Down => &mut self.s,
            Dir::Left => &mut self.e,
            Dir::Right => &mut self.w,
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
