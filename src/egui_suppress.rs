use bevy::prelude::*;
use bevy_editor_cam::controller::component::EditorCam;
use bevy_inspector_egui::bevy_egui;

pub struct EguiSupressPlugin;

impl Plugin for EguiSupressPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            suppress.before(bevy_editor_cam::input::default_camera_inputs),
        );
    }
}

fn suppress(
    mut contexts: bevy_egui::EguiContexts,
    mut editor_cams: Query<&mut EditorCam>,
    transform_gizmos: Query<&transform_gizmo_bevy::GizmoTarget>,
) {
    let ctx = contexts.ctx_mut();

    let mut should_suppress = false;

    // If menus are being interacted with, suppress
    should_suppress |= ctx.wants_pointer_input() || ctx.is_pointer_over_area();

    // Same for transform gizmos
    should_suppress |= transform_gizmos
        .iter()
        .any(|g| g.is_active() || g.is_focused());

    let enabled = !should_suppress;

    for mut cam in &mut editor_cams {
        cam.enabled_motion.pan = enabled;
        cam.enabled_motion.orbit = enabled;
        cam.enabled_motion.zoom = enabled;
    }
}
