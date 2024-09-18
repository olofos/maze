use std::marker::PhantomData;

use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, Extent3d, ShaderRef, TextureDimension, TextureFormat},
        texture::ImageSampler,
    },
    sprite::{Material2d, Material2dPlugin, Mesh2dHandle},
};

use crate::consts::{GRID_HEIGHT, GRID_WIDTH};

#[derive(Component, Reflect)]
pub struct Tilemap {
    pub grid_size: Vec2,
    pub data: Vec<u8>,
}

#[derive(Component)]
pub struct Tileset {
    pub image: Handle<Image>,
    pub num_tiles: u32,
}

pub trait TilemapData {
    fn data(&self) -> &Vec<u8>;
    fn size(&self) -> Vec4;
}

impl TilemapData for Tilemap {
    fn data(&self) -> &Vec<u8> {
        &self.data
    }

    fn size(&self) -> Vec4 {
        Vec4::new(self.grid_size.x, self.grid_size.y, 0.0, 0.0)
    }
}

pub trait TilemapMaterialShader: TypePath + Clone + Send + Sync {
    const SHADER: &'static str;
}

#[derive(TypePath, Clone)]
pub struct TilemapShader;

impl TilemapMaterialShader for TilemapShader {
    const SHADER: &'static str = "shaders/tilemap.wgsl";
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct TilemapMaterial<T: TilemapMaterialShader> {
    #[uniform(0)]
    grid_size: Vec4,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    tileset_texture: Handle<Image>,
    #[texture(3, sample_type = "u_int")]
    tilemap_texture: Handle<Image>,
    _phantom: PhantomData<T>,
}

pub fn plugin(app: &mut App) {
    app.register_type::<Tilemap>();
    register_shader::<TilemapShader>(app);
    register_data::<TilemapShader, Tilemap>(app);
}

pub fn register_shader<S: TilemapMaterialShader>(app: &mut App) {
    app.add_plugins(Material2dPlugin::<TilemapMaterial<S>>::default());
}

pub fn register_data<S: TilemapMaterialShader, T: TilemapData + Component>(app: &mut App) {
    app.add_systems(
        Update,
        (construct_materials::<S, T>, update_tilemaps::<S, T>)
            .run_if(in_state(crate::states::AppState::InGame)),
    );
}

impl Tilemap {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            grid_size: Vec2::new(width as f32, height as f32),
            data: vec![0; (width * height) as usize],
        }
    }
}

fn update_tilemaps<S: TilemapMaterialShader, T: TilemapData + Component>(
    query: Query<(&T, &Handle<TilemapMaterial<S>>), Changed<T>>,
    mut materials: ResMut<Assets<TilemapMaterial<S>>>,
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
        image.data.clone_from(tilemap.data());
    }
}

#[allow(clippy::type_complexity)]
fn construct_materials<S: TilemapMaterialShader, T: TilemapData + Component>(
    mut commands: Commands,
    query: Query<(Entity, &T, &Tileset), Without<Handle<TilemapMaterial<S>>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TilemapMaterial<S>>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, tilemap, tileset) in query.iter() {
        let mut tilemap_image = Image::new(
            Extent3d {
                width: tilemap.size().x as u32,
                height: tilemap.size().y as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            tilemap.data().clone(),
            TextureFormat::R8Uint,
            RenderAssetUsages::all(),
        );
        tilemap_image.sampler = ImageSampler::nearest();

        let tilemap_handle = images.add(tilemap_image);

        let Some(tileset_image) = images.get_mut(&tileset.image) else {
            continue;
        };
        tileset_image.reinterpret_stacked_2d_as_array(tileset.num_tiles);
        tileset_image.sampler = ImageSampler::nearest();

        let material = materials.add(TilemapMaterial::<S> {
            grid_size: tilemap.size(),
            tileset_texture: tileset.image.clone(),
            tilemap_texture: tilemap_handle,
            _phantom: PhantomData,
        });

        let mesh_handle = meshes.add(create_mesh(UVec2::new(
            GRID_WIDTH as u32,
            GRID_HEIGHT as u32,
        )));
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
            .remove::<Tileset>();
    }
}

/// The Material2d trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material2d api docs for details!
impl<T: TilemapMaterialShader + Clone + Send + Sync> Material2d for TilemapMaterial<T> {
    fn fragment_shader() -> ShaderRef {
        T::SHADER.into()
    }
}

pub fn create_mesh(size: UVec2) -> Mesh {
    let x: f32 = size.x as f32;
    let y = size.y as f32;
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
