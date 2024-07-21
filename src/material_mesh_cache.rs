use std::any::TypeId;

use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};

pub struct MaterialMeshCachePlugin;

impl Plugin for MaterialMeshCachePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TypeIdMeshCache>()
            .register_type::<UsizeMaterialCache>()
            .register_type::<UsizeColorCache>()
            .init_resource::<TypeIdMeshCache>()
            .init_resource::<UsizeMaterialCache>()
            .init_resource::<UsizeColorCache>();
    }
}

#[derive(Debug, Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
struct TypeIdMeshCache {
    cache: HashMap<TypeId, Handle<Mesh>>,
}

#[derive(Debug, Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
struct UsizeMaterialCache {
    cache: HashMap<usize, Handle<StandardMaterial>>,
}

#[derive(Debug, Default, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
struct UsizeColorCache {
    cache: HashMap<usize, Color>,
}

#[derive(SystemParam)]
pub struct MeshMaterialCache<'w> {
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    mesh_cache: ResMut<'w, TypeIdMeshCache>,

    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    material_cache: ResMut<'w, UsizeMaterialCache>,

    color_cache: ResMut<'w, UsizeColorCache>,
}

impl<'w> MeshMaterialCache<'_> {
    /// Weak handle to a default mesh of given type
    pub fn mesh<M: Meshable + Default + 'static>(&mut self) -> Handle<Mesh> {
        self.mesh_cache
            .entry(std::any::TypeId::of::<M>())
            .or_insert_with(|| self.mesh_assets.add(M::default().mesh()))
            .clone_weak()
    }

    fn material_color(&mut self, key: usize) -> (Handle<StandardMaterial>, Color) {
        if let Some(mat) = self.material_cache.get(&key) {
            let col = self.color_cache.get(&key).unwrap();

            (mat.clone_weak(), *col)
        } else {
            let color = Color::srgb_from_array(rand::random());
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

    /// Weak handle to a material from a usize.
    /// Material is unlit and of random color.
    ///
    /// If key does not exist will insert both a material and its related color
    /// into cache
    pub fn material(&mut self, key: usize) -> Handle<StandardMaterial> {
        self.material_color(key).0
    }

    /// If key does not exist will insert both a material and its related color
    /// into cache
    pub fn color(&mut self, key: usize) -> Color {
        self.material_color(key).1
    }
}
