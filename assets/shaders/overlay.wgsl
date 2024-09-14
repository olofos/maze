#import bevy_sprite::mesh2d_vertex_output::VertexOutput

// Note: This is a vec4 because on WebGPU it needs to have a size of 16
@group(2) @binding(0) var<uniform> size: vec4<f32>;
@group(2) @binding(1) var tileset_texture: texture_2d_array<f32>;
@group(2) @binding(2) var tileset_sampler: sampler;
@group(2) @binding(3) var tilemap_texture: texture_2d<u32>;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let grid_size = size.xy;
    let hex_size = size.zw;

    let tilemap_uv = vec2<u32>(mesh.uv * grid_size);
    let tile_uv_fract = fract(mesh.uv * hex_size);
    
    let tile_uv = vec2<f32>(tile_uv_fract.x, 1.0 - tile_uv_fract.y);
    let tile_index = textureLoad(tilemap_texture, tilemap_uv, 0).x;
    let hi = (tile_index >> 4) & 0xF;
    let lo = tile_index & 0xF;

    let subtile_uv = floor(mesh.uv * hex_size) - floor(mesh.uv * grid_size) * hex_size / grid_size;
    var index: u32;

    if subtile_uv.x == 1 && subtile_uv.y == 1 {
        index = hi;
    } else if subtile_uv.x == 2 && subtile_uv.y == 1 {
        index = lo;
    } else {
        discard;
    }
        return textureSample(tileset_texture, tileset_sampler, tile_uv, index);
}
