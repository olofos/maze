use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
};

use crate::{components::*, consts::*, TilemapMaterial};

pub fn update_tilemaps(
    mut grid_query: Query<&mut Grid>,
    mut images: ResMut<Assets<Image>>,
    tilemap_query: Query<&crate::Tilemap>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
) {
    let Ok(tilemap) = tilemap_query.get_single() else {
        return;
    };

    let grid = grid_query.single_mut();

    let mut data = Vec::with_capacity(GRID_WIDTH * GRID_HEIGHT);

    for y in 0..GRID_HEIGHT {
        for x in 0..GRID_WIDTH {
            if grid.visited[y * GRID_WIDTH + x] == 0
                && !(x == 0 && y == 0)
                && !(x == GRID_WIDTH - 1 && y == GRID_HEIGHT - 1)
            {
                data.push(17)
            } else {
                let mut val = 0b1111;
                if grid.walls[y * GRID_WIDTH + x].n || y == GRID_HEIGHT - 1 {
                    val &= !0b0001;
                }
                if grid.walls[y * GRID_WIDTH + x].w || x == GRID_WIDTH - 1 {
                    val &= !0b0010;
                }
                if grid.walls[y * GRID_WIDTH + x].s || y == 0 {
                    val &= !0b0100;
                }
                if grid.walls[y * GRID_WIDTH + x].e || x == 0 {
                    val &= !0b1000;
                }

                data.push(val);
            }
        }
    }

    let mat = materials.get_mut(&tilemap.material).unwrap();

    let mut tilemap_image = Image::new(
        Extent3d {
            width: GRID_WIDTH as u32,
            height: GRID_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::R8Uint,
        RenderAssetUsages::all(),
    );
    tilemap_image.sampler = ImageSampler::nearest();

    let tilemap_handle = images.add(tilemap_image);
    mat.tilemap_texture = Some(tilemap_handle);
}
