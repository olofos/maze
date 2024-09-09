#import bevy_sprite::mesh2d_vertex_output::VertexOutput
// we can import items from shader modules in the assets folder with a quoted path

@group(2) @binding(0) var<uniform> grid_size: vec2<f32>; 
@group(2) @binding(1) var tileset_texture: texture_2d_array<f32>;
@group(2) @binding(2) var tileset_sampler: sampler;
@group(2) @binding(3) var tilemap_texture: texture_2d<u32>;

const GRID_WIDTH: f32 = 4.0;
const GRID_HEIGHT: f32 = 4.0;
const NUM_TILES: f32 = 16.0;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let tilemap_uv = mesh.uv * grid_size;
    let tile_uv = fract(mesh.uv * grid_size);
    let tile_index = textureLoad(tilemap_texture, vec2<u32>(tilemap_uv), 0).x;
    return textureSample(tileset_texture, tileset_sampler, tile_uv, tile_index);
}
