use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::TilemapMaterial;

#[derive(Component)]
pub struct Tilemap {
    material: Handle<TilemapMaterial>,
    tileset: Handle<Image>,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    z: f32,
}

impl Tilemap {
    pub fn new(tileset: Handle<Image>, width: u32, height: u32, z: f32) -> Self {
        Self {
            material: Handle::default(),
            tileset,
            width,
            height,
            data: vec![0; (width * height) as usize],
            z,
        }
    }
}

pub fn update_tilemaps(
    mut query: Query<&mut Tilemap, Changed<Tilemap>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TilemapMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for mut tilemap in query.iter_mut() {
        let mut tilemap_image = Image::new(
            Extent3d {
                width: tilemap.width as u32,
                height: tilemap.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            tilemap.data.clone(),
            TextureFormat::R8Uint,
            RenderAssetUsages::all(),
        );
        tilemap_image.sampler = ImageSampler::nearest();

        let tilemap_handle = images.add(tilemap_image);

        if let Some(material) = materials.get_mut(&tilemap.material) {
            material.tilemap_texture = Some(tilemap_handle.clone());
        } else {
            let material = materials.add(crate::TilemapMaterial {
                grid_size: Vec4::new(tilemap.width as f32, tilemap.height as f32, 0.0, 0.0),
                tileset_texture: Some(tilemap.tileset.clone()),
                tilemap_texture: Some(tilemap_handle),
            });
            tilemap.material = material.clone();

            let mesh_handle = meshes.add(crate::create_mesh());
            let mesh: Mesh2dHandle = mesh_handle.into();

            commands.spawn((MaterialMesh2dBundle {
                mesh,
                transform: Transform::default().with_translation(Vec3::new(0.0, 0.0, tilemap.z)),
                material,
                ..default()
            },));
        }
    }
}
