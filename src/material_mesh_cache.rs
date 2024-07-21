use std::any::TypeId;

use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};

pub struct MaterialMeshCachePlugin;

impl Plugin for MaterialMeshCachePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TypeIdMeshCache>()
            .register_type::<MaterialsCache>()
            .register_type::<ColorCache>()
            .init_resource::<TypeIdMeshCache>()
            .init_resource::<MaterialsCache>()
            .init_resource::<ColorCache>();
    }
}

#[derive(Debug, Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
struct TypeIdMeshCache {
    cache: HashMap<TypeId, Handle<Mesh>>,
}

#[derive(Debug, Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
struct MaterialsCache {
    cache: HashMap<MaterialKey, Handle<StandardMaterial>>,
}

#[derive(Debug, Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
struct ColorCache {
    cache: HashMap<MaterialKey, Color>,
}

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Reflect, Clone, Copy)]
pub enum MaterialKey {
    Usize(usize),
    LinearRgba([u8; 4]),
}

impl From<[u8; 4]> for MaterialKey {
    fn from(v: [u8; 4]) -> Self {
        Self::LinearRgba(v)
    }
}

impl From<usize> for MaterialKey {
    fn from(v: usize) -> Self {
        Self::Usize(v)
    }
}

#[derive(SystemParam)]
pub struct MeshMaterialCache<'w> {
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    mesh_cache: ResMut<'w, TypeIdMeshCache>,

    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    material_cache: ResMut<'w, MaterialsCache>,

    color_cache: ResMut<'w, ColorCache>,
}

impl<'w> MeshMaterialCache<'_> {
    /// Weak handle to a default mesh of given type
    pub fn mesh<M: Meshable + Default + 'static>(&mut self) -> Handle<Mesh> {
        self.mesh_cache
            .entry(std::any::TypeId::of::<M>())
            .or_insert_with(|| self.mesh_assets.add(M::default().mesh()))
            .clone_weak()
    }

    fn material_color(&mut self, key: MaterialKey) -> (Handle<StandardMaterial>, Color) {
        if let Some(mat) = self.material_cache.get(&key) {
            let col = self.color_cache.get(&key).unwrap();

            (mat.clone_weak(), *col)
        } else {
            let color = match key {
                MaterialKey::Usize(_) => Color::srgb_from_array(rand::random()),
                MaterialKey::LinearRgba(color) => {
                    Color::LinearRgba(LinearRgba::from_u8_array(color))
                }
            };
            self.color_cache.insert(key, color);
            let mut smat: StandardMaterial = color.into();

            smat.unlit = true;
            smat.cull_mode = None;
            let handle = self.material_assets.add(smat);
            let weak = handle.clone_weak();
            self.material_cache.insert(key, handle);

            (weak, color)
        }
    }

    pub fn material(&mut self, key: impl Into<MaterialKey>) -> Handle<StandardMaterial> {
        self.material_color(key.into()).0
    }

    pub fn color(&mut self, key: impl Into<MaterialKey>) -> Color {
        self.material_color(key.into()).1
    }
}
