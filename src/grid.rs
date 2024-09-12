use crate::{consts::*, tilemap};
use bevy::prelude::*;

#[derive(Component)]
pub struct Grid {
    pub data: Vec<u8>,
}

impl tilemap::TilemapData for Grid {
    fn data(&self) -> &Vec<u8> {
        &self.data
    }

    fn grid_size(&self) -> UVec2 {
        UVec2::new(GRID_WIDTH as u32, GRID_HEIGHT as u32)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    North = 0b0001,
    East = 0b0010,
    South = 0b0100,
    West = 0b1000,
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

impl From<Dir> for IVec2 {
    fn from(dir: Dir) -> Self {
        match dir {
            Dir::North => IVec2::Y,
            Dir::East => IVec2::X,
            Dir::South => -IVec2::Y,
            Dir::West => -IVec2::X,
        }
    }
}

impl Grid {
    pub fn get(&self, pos: IVec2) -> u8 {
        self.data[(pos.y as usize) * GRID_WIDTH + pos.x as usize]
    }

    pub fn get_mut(&mut self, pos: IVec2) -> &mut u8 {
        &mut self.data[(pos.y as usize) * GRID_WIDTH + pos.x as usize]
    }

    pub fn is_visited(&self, pos: IVec2) -> bool {
        self.get(pos) > 0
    }

    pub fn remove_wall(&mut self, pos: IVec2, dir: Dir) {
        *self.get_mut(pos) |= dir as u8;
        let pos: IVec2 = pos + IVec2::from(dir);
        *self.get_mut(pos) |= dir.reverse() as u8;
    }

    pub fn has_wall(&self, pos: IVec2, dir: Dir) -> bool {
        self.get(pos) & (dir as u8) == 0
    }
}
