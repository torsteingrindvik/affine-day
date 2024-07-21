use bevy::{prelude::*, window::PrimaryWindow};
use bevy_inspector_egui::{
    bevy_egui::{EguiContext, EguiSettings},
    bevy_inspector::ui_for_resource,
    egui,
    inspector_options::ReflectInspectorOptions,
    quick::WorldInspectorPlugin,
    InspectorOptions,
};

use crate::{gizmos::GizmoSettings, ImagePlanes, ImagePoints, ImageSize};

/// Combine relevant resources into one place
pub struct UiSettingsPlugin;

#[derive(Debug, Resource, Reflect, PartialEq, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
pub struct UiSettings {
    show_world_ui: bool,

    #[inspector(min = 0.5, max = 1.5)]
    ui_scale: f32,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_world_ui: false,
            ui_scale: 1.0,
        }
    }
}

impl Plugin for UiSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiSettings>()
            .register_type::<UiSettings>()
            .add_systems(
                Update,
                (ui, set_scale.run_if(resource_changed::<UiSettings>)),
            )
            .add_plugins(
                WorldInspectorPlugin::new()
                    .run_if(|settings: Res<UiSettings>| settings.show_world_ui),
            );
    }
}

fn ui(world: &mut World) {
    let Ok(egui_context) = world
        .query_filtered::<&mut EguiContext, With<PrimaryWindow>>()
        .get_single(world)
    else {
        return;
    };
    let mut egui_context = egui_context.clone();

    egui::Window::new("Settings").show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            ui_for_resource::<ImagePlanes>(world, ui);
            ui_for_resource::<ImagePoints>(world, ui);
            ui_for_resource::<ImageSize>(world, ui);
            ui_for_resource::<GizmoSettings>(world, ui);
            ui_for_resource::<UiSettings>(world, ui);
        });
    });
}

fn set_scale(mut commands: Commands, settings: Res<UiSettings>) {
    commands.insert_resource(EguiSettings {
        scale_factor: settings.ui_scale,
        ..default()
    });
}
