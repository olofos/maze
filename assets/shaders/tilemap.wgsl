#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Note: This is a vec4 because on WebGPU it needs to have a size of 16
@group(2) @binding(0) var<uniform> grid_size: vec4<f32>;
@group(2) @binding(1) var tileset_texture: texture_2d_array<f32>;
@group(2) @binding(2) var tileset_sampler: sampler;
@group(2) @binding(3) var tilemap_texture: texture_2d<u32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let size = grid_size.xy;
    let tilemap_uv = mesh.uv * size;
    let tile_uv_fract = fract(mesh.uv * size);
    let tile_uv = vec2<f32>(tile_uv_fract.x, 1.0 - tile_uv_fract.y);
    let tile_index = textureLoad(tilemap_texture, vec2<u32>(tilemap_uv), 0).x;
    return textureSample(tileset_texture, tileset_sampler, tile_uv, tile_index);
}
