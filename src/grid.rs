use crate::{consts::*, tilemap};
use bevy::prelude::*;

#[derive(Component)]
pub struct Grid {
    data: Vec<u8>,
    pub region: Vec<u16>,
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

impl tilemap::TilemapData for Grid {
    fn data(&self) -> &Vec<u8> {
        &self.data
    }

    fn size(&self) -> Vec4 {
        Vec4::new(GRID_WIDTH as f32, GRID_HEIGHT as f32, 0.0, 0.0)
    }
}

impl Grid {
    pub fn new() -> Self {
        Self {
            data: vec![0; GRID_WIDTH * GRID_HEIGHT],
            region: (0..(GRID_WIDTH * GRID_HEIGHT)).map(|n| n as u16).collect(),
        }
    }

    fn index(&self, pos: IVec2) -> usize {
        (pos.y as usize) * GRID_WIDTH + pos.x as usize
    }

    pub fn region(&self, pos: IVec2) -> u16 {
        self.region[self.index(pos)]
    }

    pub fn get_walls(&self, pos: IVec2) -> u8 {
        self.data[self.index(pos)]
    }

    pub fn get_walls_mut(&mut self, pos: IVec2) -> &mut u8 {
        let index = self.index(pos);
        &mut self.data[index]
    }

    pub fn is_visited(&self, pos: IVec2) -> bool {
        self.get_walls(pos) > 0
    }

    pub fn remove_wall(&mut self, pos: IVec2, dir: Dir) -> Result<(), ()> {
        let new_pos: IVec2 = pos + IVec2::from(dir);
        if self.region[self.index(pos)] == self.region[self.index(new_pos)] {
            return Err(());
        }
        *self.get_walls_mut(pos) |= dir as u8;
        let region = self.region[self.index(pos)];
        *self.get_walls_mut(new_pos) |= dir.reverse() as u8;
        self.fill_region(new_pos, region);

        Ok(())
    }

    fn fill_region(&mut self, pos: IVec2, region: u16) {
        let index = self.index(pos);
        if self.region[index] == region {
            return;
        }
        self.region[index] = region;
        for dir in [Dir::North, Dir::East, Dir::South, Dir::West] {
            if !self.has_wall(pos, dir) {
                self.fill_region(pos + IVec2::from(dir), region);
            }
        }
    }

    pub fn has_wall(&self, pos: IVec2, dir: Dir) -> bool {
        (self.get_walls(pos) & (dir as u8) == 0)
            || (pos.x == 0 && dir == Dir::West)
            || (pos.y == 0 && dir == Dir::South)
            || (pos.x == GRID_WIDTH as i32 - 1 && dir == Dir::East)
            || (pos.y == GRID_HEIGHT as i32 - 1 && dir == Dir::North)
    }
}
