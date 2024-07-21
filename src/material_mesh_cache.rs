use std::any::TypeId;

use bevy::{ecs::system::SystemParam, prelude::*, utils::HashMap};

pub struct MaterialMeshCachePlugin;

impl Plugin for MaterialMeshCachePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TypeIdMeshCache>()
            .register_type::<UsizeMaterialCache>()
            .init_resource::<TypeIdMeshCache>()
            .init_resource::<UsizeMaterialCache>();
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

#[derive(SystemParam)]
pub struct MeshMaterialCache<'w> {
    mesh_assets: ResMut<'w, Assets<Mesh>>,
    mesh_cache: ResMut<'w, TypeIdMeshCache>,

    material_assets: ResMut<'w, Assets<StandardMaterial>>,
    material_cache: ResMut<'w, UsizeMaterialCache>,
}

impl<'w> MeshMaterialCache<'_> {
    /// Weak handle to a default mesh of given type
    pub fn mesh<M: Meshable + Default + 'static>(&mut self) -> Handle<Mesh> {
        self.mesh_cache
            .entry(std::any::TypeId::of::<M>())
            .or_insert_with(|| self.mesh_assets.add(M::default().mesh()))
            .clone_weak()
    }

    /// Weak handle to a material from a usize.
    /// Material is unlit and of random color.
    pub fn material(&mut self, key: usize) -> Handle<StandardMaterial> {
        self.material_cache
            .entry(key)
            .or_insert_with(|| {
                let color = Color::srgb_from_array(rand::random());
                let mut smat: StandardMaterial = color.into();

                smat.unlit = true;
                smat.cull_mode = None;
                self.material_assets.add(smat)
            })
            .clone_weak()
    }
}
