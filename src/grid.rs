use crate::{consts::*, disjoint_set::DisjointSet, tilemap};
use bevy::prelude::*;

#[derive(Component)]
pub struct Grid {
    data: Vec<u8>,
    pub regions: DisjointSet,
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
            regions: DisjointSet::new(GRID_WIDTH * GRID_HEIGHT),
        }
    }

    fn index(&self, pos: IVec2) -> usize {
        (pos.y as usize) * GRID_WIDTH + pos.x as usize
    }

    pub fn region(&self, pos: IVec2) -> usize {
        self.regions.find(self.index(pos))
    }

    pub fn join_regions(&mut self, pos: IVec2, other: IVec2) {
        self.regions.join(self.index(pos), self.index(other));
    }

    pub fn get_walls(&self, pos: IVec2) -> u8 {
        self.data[self.index(pos)]
    }

    pub fn get_walls_mut(&mut self, pos: IVec2) -> &mut u8 {
        let index = self.index(pos);
        &mut self.data[index]
    }

    pub fn is_visited(&self, pos: IVec2) -> bool {
        !self.regions.is_singleton(self.index(pos))
    }

    pub fn remove_wall(&mut self, pos: IVec2, dir: Dir) -> Result<(), ()> {
        let new_pos: IVec2 = pos + IVec2::from(dir);
        if self.region(pos) == self.region(new_pos) {
            return Err(());
        }
        self.join_regions(pos, new_pos);

        *self.get_walls_mut(pos) |= dir as u8;
        *self.get_walls_mut(new_pos) |= dir.reverse() as u8;

        Ok(())
    }

    pub fn has_wall(&self, pos: IVec2, dir: Dir) -> bool {
        (self.get_walls(pos) & (dir as u8) == 0)
            || (pos.x == 0 && dir == Dir::West)
            || (pos.y == 0 && dir == Dir::South)
            || (pos.x == GRID_WIDTH as i32 - 1 && dir == Dir::East)
            || (pos.y == GRID_HEIGHT as i32 - 1 && dir == Dir::North)
    }

    fn is_inside(&self, pos: IVec2) -> bool {
        pos.x >= 0 && pos.x < GRID_WIDTH as i32 && pos.y >= 0 && pos.y < GRID_HEIGHT as i32
    }

    pub fn possible_moves(&self, pos: IVec2) -> Vec<Dir> {
        [Dir::North, Dir::East, Dir::South, Dir::West]
            .into_iter()
            .filter(|d| {
                let p = pos + IVec2::from(*d);
                self.is_inside(p) && self.region(p) != self.region(pos)
            })
            .collect()
    }
}
