use bevy::prelude::*;

use crate::consts::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Goal;

#[derive(Debug, Clone, Reflect)]
pub struct Walls {
    pub n: bool,
    pub e: bool,
    pub s: bool,
    pub w: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    North,
    South,
    West,
    East,
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
            w: true,
            e: true,
        }
    }
}

impl Walls {
    pub fn get_mut(&mut self, dir: Dir) -> &mut bool {
        match dir {
            Dir::North => &mut self.n,
            Dir::South => &mut self.s,
            Dir::West => &mut self.w,
            Dir::East => &mut self.e,
        }
    }
}

impl Dir {
    pub fn reverse(&self) -> Self {
        match self {
            Dir::North => Dir::South,
            Dir::South => Dir::North,
            Dir::West => Dir::East,
            Dir::East => Dir::West,
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
