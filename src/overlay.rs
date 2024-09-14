use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    sprite::{Material2d, Mesh2dHandle},
};

use crate::consts::*;

#[derive(Component, Reflect)]
pub struct Overlay {
    pub data: Vec<u8>,
    pub tileset: Handle<Image>,
}

impl Overlay {
    pub fn new(tileset: Handle<Image>) -> Self {
        Self {
            data: vec![0; GRID_WIDTH * GRID_HEIGHT],
            tileset,
        }
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct OverlayMaterial {
    #[uniform(0)]
    size: Vec4,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    tileset_texture: Handle<Image>,
    #[texture(3, sample_type = "u_int")]
    tilemap_texture: Handle<Image>,
}

pub fn update_overlays(
    query: Query<(&Overlay, &Handle<OverlayMaterial>), Changed<Overlay>>,
    mut materials: ResMut<Assets<OverlayMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (tilemap, material) in query.iter() {
        // This needs to be get mut to signal that the material has changed
        let Some(material) = materials.get_mut(material) else {
            continue;
        };
        let Some(image) = images.get_mut(&material.tilemap_texture) else {
            continue;
        };
        image.data.clone_from(&tilemap.data);
    }
}

pub fn construct_materials(
    mut commands: Commands,
    query: Query<(Entity, &Overlay), Without<Handle<OverlayMaterial>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<OverlayMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, overlay) in query.iter() {
        let mut overlay_image = Image::new(
            Extent3d {
                width: GRID_WIDTH as u32,
                height: GRID_HEIGHT as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            overlay.data.clone(),
            TextureFormat::R8Uint,
            RenderAssetUsages::all(),
        );
        overlay_image.sampler = ImageSampler::nearest();

        let tilemap_handle = images.add(overlay_image);

        let Some(tileset_image) = images.get_mut(&overlay.tileset) else {
            continue;
        };
        tileset_image.reinterpret_stacked_2d_as_array(17);
        tileset_image.sampler = ImageSampler::nearest();

        let material = materials.add(OverlayMaterial {
            size: Vec4::new(
                GRID_WIDTH as f32,
                GRID_HEIGHT as f32,
                PLAYFIELD_WIDTH as f32 / 16.0,
                PLAYFIELD_HEIGHT as f32 / 16.0,
            ),
            tileset_texture: overlay.tileset.clone(),
            tilemap_texture: tilemap_handle,
        });

        let mesh_handle = meshes.add(crate::tilemap::create_mesh(UVec2::new(
            GRID_WIDTH as u32,
            GRID_HEIGHT as u32,
        )));
        let mesh: Mesh2dHandle = mesh_handle.into();

        commands.entity(entity).insert((
            mesh,
            material,
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));
    }
}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl Material2d for OverlayMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/overlay.wgsl".into()
    }
}
