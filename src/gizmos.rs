use bevy::prelude::*;

pub struct GizmosPlugin;

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct GizmoSettings {
    show_world_axes: bool,
}

impl Plugin for GizmosPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GizmoSettings>()
            .init_resource::<GizmoSettings>()
            .add_systems(Update, gizmo_world_axes);
    }
}

fn gizmo_world_axes(mut gizmos: Gizmos, settings: Res<GizmoSettings>) {
    if settings.show_world_axes {
        gizmos.axes(Transform::default(), 1.0);
    }
}
