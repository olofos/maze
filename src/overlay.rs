use bevy::prelude::*;

use crate::{
    consts::*,
    tilemap::{TilemapData, TilemapMaterialShader},
};

#[derive(Component, Reflect)]
pub struct Overlay {
    pub data: Vec<u8>,
}

#[derive(TypePath, Clone)]
pub struct OverlayShader;

pub fn plugin(app: &mut App) {
    app.register_type::<Overlay>().add_plugins((
        crate::tilemap::register_shader::<OverlayShader>,
        crate::tilemap::register_data::<OverlayShader, Overlay>,
    ));
}

impl TilemapMaterialShader for OverlayShader {
    const SHADER: &'static str = "shaders/overlay.wgsl";
}

impl TilemapData for Overlay {
    fn data(&self) -> &Vec<u8> {
        &self.data
    }

    fn size(&self) -> Vec4 {
        Vec4::new(
            GRID_WIDTH as f32,
            GRID_HEIGHT as f32,
            PLAYFIELD_WIDTH / 16.0,
            PLAYFIELD_HEIGHT / 16.0,
        )
    }
}

impl Overlay {
    pub fn new() -> Self {
        Self {
            data: vec![0; GRID_WIDTH * GRID_HEIGHT],
        }
    }
}
