use bevy::prelude::*;

use crate::{
    material_mesh_cache::MeshMaterialCache, ImagePlane, ImagePlanes, ImagePoint, ImagePointIndex,
    ImagePoints, MainImagePlane, MoveOverFirstPlaneEvent, SecondaryCamera,
};

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
            .add_systems(
                Update,
                (gizmo_world_axes, gizmo_point_rays, gizmo_1st_image_plane),
            );
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

fn gizmo_1st_image_plane(
    mut stored_pos: Local<Option<Vec3>>,
    mut gizmos: Gizmos,
    point_settings: Res<ImagePoints>,
    mut over_events: EventReader<MoveOverFirstPlaneEvent>,
    secondary_camera: Query<&Camera, With<SecondaryCamera>>,
    main_image_plane: Query<Entity, (With<ImagePlane>, With<MainImagePlane>)>,
) {
    for e in over_events.read() {
        // We only care about interactions from this camera
        if secondary_camera.get(e.hit.camera).is_err() {
            continue;
        }

        // Only care about this exact plane
        if main_image_plane.get(e.data.target).is_err() {
            continue;
        }

        let Some(pos) = e.hit.position else {
            warn!("unexpected positionless main image plane hit");
            continue;
        };

        // All hit positions should be at image plane Z=1.0
        if (1.0 - pos.z).abs() > 0.001 {
            warn!("unexpected main image plane world position: {pos:?}");
            continue;
        }

        debug!("hit image plane at: {:?}", pos.xy());
        *stored_pos = Some(pos);
    }

    if let Some(pos) = *stored_pos {
        gizmos.sphere(
            pos,
            Quat::default(),
            point_settings.point_size / 2.,
            Color::WHITE.with_alpha(0.15),
        );
    }
}
