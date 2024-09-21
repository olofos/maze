use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Goal;

#[derive(Component)]
pub struct Trees;

#[derive(Component)]
pub struct Ground;

#[derive(Component, Default)]
pub struct Cover(f32);

impl Cover {
    pub fn step(&mut self, dt: f32) -> u8 {
        self.0 += dt * 60.0;
        let inc = self.0.floor();
        self.0 -= inc;
        inc as u8
    }
}
