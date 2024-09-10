use bevy::{
    prelude::Image,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension},
        texture::ImageSampler,
    },
};

use crate::consts::*;

#[derive(Clone, Copy)]
enum SubTile {
    Full = 0,
    N = 1,
    E = 2,
    S = 3,
    W = 4,
    SW = 5,
    NW = 6,
    NE = 7,
    SE = 8,
    CornerNW = 9,
    CornerNE = 10,
    CornerSE = 11,
    CornerSW = 12,
    Empty = 13,
}

const SUBTILE_WIDTH: usize = 16;
const SUBTILE_HEIGHT: usize = 16;
const CHANNELS: usize = 4;
const NUM_TILES: usize = 17;

fn blit_tile(
    src: &[u8],
    dst: &mut [u8],
    src_tile_num: SubTile,
    dst_tile_num: usize,
    pos: (usize, usize),
) {
    let src_tile_num = src_tile_num as usize;
    let (pos_x, pos_y) = pos;
    let dst = &mut dst[dst_tile_num * TILE_WIDTH * TILE_HEIGHT * CHANNELS..];
    let src = &src[(src_tile_num) * (CHANNELS * SUBTILE_WIDTH * SUBTILE_HEIGHT)
        ..(src_tile_num + 1) * (CHANNELS * SUBTILE_WIDTH * SUBTILE_HEIGHT)];
    let dst = &mut dst
        [(pos_y * SUBTILE_HEIGHT * CHANNELS * TILE_WIDTH + CHANNELS * SUBTILE_WIDTH * pos_x)..];

    for y in 0..SUBTILE_HEIGHT {
        for x in 0..SUBTILE_WIDTH {
            for i in 0..CHANNELS {
                dst[CHANNELS * TILE_WIDTH * y + CHANNELS * x + i] =
                    src[(CHANNELS * SUBTILE_WIDTH * (SUBTILE_HEIGHT - y - 1)) + CHANNELS * x + i];
            }
        }
    }
}

pub fn expand(image: &Image) -> Image {
    use SubTile::*;

    let max_x = TILE_WIDTH / SUBTILE_WIDTH - 1;
    let max_y = TILE_HEIGHT / SUBTILE_HEIGHT - 1;

    let mut tiles = vec![0u8; TILE_WIDTH * TILE_HEIGHT * NUM_TILES * CHANNELS];

    let src = &image.data;
    let dst = &mut tiles;

    for x in 0..=max_x {
        for y in 0..=max_y {
            blit_tile(src, dst, Full, NUM_TILES - 1, (x, y));
        }
    }

    for dst_tile_num in 0..(NUM_TILES - 1) {
        let ne = [NE, E, N, CornerNE][dst_tile_num & 0b0011];
        let se = [SE, S, E, CornerSE][(dst_tile_num & 0b0110) >> 1];
        let sw = [SW, W, S, CornerSW][(dst_tile_num & 0b1100) >> 2];
        let nw = [NW, N, W, CornerNW][(dst_tile_num & 0b1000) >> 3 | (dst_tile_num & 0b0001) << 1];
        let n = [N, Empty][dst_tile_num & 0b001];
        let e = [E, Empty][(dst_tile_num & 0b010) >> 1];
        let s = [S, Empty][(dst_tile_num & 0b100) >> 2];
        let w = [W, Empty][(dst_tile_num & 0b1000) >> 3];

        blit_tile(src, dst, sw, dst_tile_num, (0, 0));
        blit_tile(src, dst, se, dst_tile_num, (max_x, 0));
        blit_tile(src, dst, nw, dst_tile_num, (0, max_y));
        blit_tile(src, dst, ne, dst_tile_num, (max_x, max_y));

        for x in 1..max_x {
            blit_tile(src, dst, s, dst_tile_num, (x, 0));
            blit_tile(src, dst, n, dst_tile_num, (x, max_y));
        }

        for y in 1..max_y {
            blit_tile(src, dst, w, dst_tile_num, (0, y));
            blit_tile(src, dst, e, dst_tile_num, (max_x, y));
        }

        for x in 1..max_x {
            for y in 1..max_y {
                blit_tile(src, dst, Empty, dst_tile_num, (x, y));
            }
        }
    }

    let mut tileset_image = Image::new(
        Extent3d {
            width: TILE_WIDTH as u32,
            height: (TILE_HEIGHT * NUM_TILES) as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        tiles,
        image.texture_descriptor.format,
        RenderAssetUsages::all(),
    );
    tileset_image.sampler = ImageSampler::nearest();

    tileset_image
}
