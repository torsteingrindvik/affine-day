use bevy::prelude::*;

use crate::{material_mesh_cache::MeshMaterialCache, ImagePlanes, ImagePoint, ImagePointIndex};

pub struct GizmosPlugin;

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct GizmoSettings {
    show_world_axes: bool,
    show_point_rays: bool,
}

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GizmoSettings>()
            .init_resource::<GizmoSettings>()
            .add_systems(Update, (gizmo_world_axes, gizmo_point_rays));
    }
}

fn gizmo_world_axes(mut gizmos: Gizmos, settings: Res<GizmoSettings>) {
    if settings.show_world_axes {
        gizmos.axes(Transform::default(), 1.0);
    }
}

fn gizmo_point_rays(
    mut gizmos: Gizmos,
    mut cache: MeshMaterialCache,
    settings: Res<GizmoSettings>,
    planes: Res<ImagePlanes>,
    points: Query<(&Transform, &ImagePointIndex), With<ImagePoint>>,
) {
    let num_planes = planes.num_planes as f32;
    if settings.show_point_rays {
        for (transform, ImagePointIndex { index }) in &points {
            gizmos.line(
                transform.translation,
                transform.translation * num_planes,
                cache.color(*index),
            );
        }
    }
}
