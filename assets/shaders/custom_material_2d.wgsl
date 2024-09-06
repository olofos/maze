#import bevy_sprite::mesh2d_vertex_output::VertexOutput
// we can import items from shader modules in the assets folder with a quoted path

@group(2) @binding(1) var tileset_texture: texture_2d<f32>;
@group(2) @binding(2) var tileset_sampler: sampler;
@group(2) @binding(3) var tilemap_texture: texture_2d<f32>;
@group(2) @binding(4) var tilemap_sampler: sampler;

const GRID_WIDTH: f32 = 4.0;
const GRID_HEIGHT: f32 = 4.0;
const NUM_TILES: f32 = 16.0;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let tilemap_uv = floor(mesh.uv * vec2<f32>(GRID_WIDTH,GRID_HEIGHT)) / vec2<f32>(GRID_WIDTH,GRID_HEIGHT);
    let tile_uv = fract(mesh.uv * vec2<f32>(GRID_WIDTH,GRID_HEIGHT)) / vec2<f32>(1.0, NUM_TILES);
    // let tilemap_uv = floor(mesh.uv) /4.0;
    // let tile_uv = fract(mesh.uv) * vec2<f32>(1.0, 1.0/NUM_TILES);
    let tile_index = textureSample(tilemap_texture, tilemap_sampler, tilemap_uv).yx * NUM_TILES;

    return textureSample(tileset_texture, tileset_sampler, tile_uv + tile_index);
    // return textureSample(tileset_texture, tileset_sampler, mesh.uv);

    // let tilemap_uv = vec2<u32>(floor(mesh.uv * vec2<f32>(GRID_WIDTH,GRID_HEIGHT)) / vec2<f32>(GRID_WIDTH,GRID_HEIGHT) * 16.0);
    // let tile_uv = vec2<u32>(fract(mesh.uv) * vec2<f32>(1.0, 1.0/NUM_TILES) * 16.0);
    // return textureLoad(tileset_texture, tilemap_uv + tile_uv, 0);
}
