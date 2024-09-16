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
        crate::tilemap::plugin_with_shader::<OverlayShader>,
        crate::tilemap::plugin_with_data::<OverlayShader, Overlay>,
    ));
}

impl TilemapMaterialShader for OverlayShader {
    const SHADER: &'static str = "shaders/overlay.wgsl";
}

impl TilemapData for Overlay {
    fn data(&self) -> &Vec<u8> {
        &self.data
    }

    fn grid_size(&self) -> UVec2 {
        UVec2::new(GRID_WIDTH as u32, GRID_HEIGHT as u32)
    }

    fn subgrid_size(&self) -> UVec2 {
        UVec2::new(
            (PLAYFIELD_WIDTH / 16.0) as u32,
            (PLAYFIELD_HEIGHT / 16.0) as u32,
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
