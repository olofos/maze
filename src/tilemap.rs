use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    sprite::{Material2d, Mesh2dHandle},
};

use crate::consts::{GRID_HEIGHT, GRID_WIDTH};

#[derive(Component)]
pub struct Tilemap {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TilemapMaterial {
    #[uniform(0)]
    grid_size: Vec4,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    tileset_texture: Handle<Image>,
    #[texture(3, sample_type = "u_int")]
    tilemap_texture: Handle<Image>,
}

impl Tilemap {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0; (width * height) as usize],
        }
    }
}

pub fn update_tilemaps(
    query: Query<(&Tilemap, &Handle<TilemapMaterial>), Changed<Tilemap>>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (tilemap, material) in query.iter() {
        // This needs to be get mut to signal that the material has changed
        let Some(material) = materials.get_mut(material) else {
            continue;
        };
        let tilemap_handle = material.tilemap_texture.clone();
        let Some(image) = images.get_mut(&tilemap_handle) else {
            continue;
        };
        image.data.clone_from(&tilemap.data);
    }
}

pub fn construct_materials(
    mut commands: Commands,
    query: Query<(Entity, &Tilemap, &Handle<Image>), Without<Handle<TilemapMaterial>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, tilemap, tileset) in query.iter() {
        let mut tilemap_image = Image::new(
            Extent3d {
                width: tilemap.width,
                height: tilemap.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            tilemap.data.clone(),
            TextureFormat::R8Uint,
            RenderAssetUsages::all(),
        );
        tilemap_image.sampler = ImageSampler::nearest();

        let tilemap_handle = images.add(tilemap_image);

        let material = materials.add(TilemapMaterial {
            grid_size: Vec4::new(tilemap.width as f32, tilemap.height as f32, 0.0, 0.0),
            tileset_texture: tileset.clone(),
            tilemap_texture: tilemap_handle,
        });

        let mesh_handle = meshes.add(create_mesh());
        let mesh: Mesh2dHandle = mesh_handle.into();

        commands
            .entity(entity)
            .insert((
                mesh,
                material,
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
            ))
            .remove::<Handle<Image>>();
    }
}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for TilemapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tilemap.wgsl".into()
    }
}

fn create_mesh() -> Mesh {
    let x = GRID_WIDTH as f32;
    let y = GRID_HEIGHT as f32;
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[0.0, 0.0, 0.0], [0.0, y, 0.0], [x, y, 0.0], [x, 0.0, 0.0]],
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, {
        vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
    })
    .with_inserted_indices(Indices::U32(vec![0, 1, 2, 2, 3, 0]))
}
