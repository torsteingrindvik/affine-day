use bevy::{prelude::*, render::camera::Viewport, window::PrimaryWindow};

pub struct ViewportCameraPlugin;

#[derive(Debug)]
pub enum Anchor {
    TopLeft,
    BottomRight,
}

#[derive(Debug, Component)]
pub struct ViewportCamera {
    pub anchor: Anchor,

    /// Width, height fraction in (0.0, 1.0) range of the full size window
    pub fraction: Vec2,
}

impl Plugin for ViewportCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, add_viewport_camera);
    }
}

fn add_viewport_camera(
    window: Query<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(&mut Camera, &ViewportCamera)>,
) {
    let Ok(win) = window.get_single() else {
        warn!("unexpected no primary window");
        return;
    };

    for (mut cam, vcam) in &mut cameras {
        let size = win.resolution.physical_size();

        assert!(
            vcam.fraction.cmpgt(Vec2::ZERO).all(),
            "both dimensions of the fraction must be greater than 0.0"
        );

        // let fractional_size: UVec2 = (size * vcam.fraction).as_uvec2();
        let fractional_size = (size.as_vec2() * vcam.fraction).as_uvec2();
        assert!(
            size.cmpge(fractional_size).all(),
            "both dimensions of the fraction must be less than 1.0"
        );

        cam.viewport = Some(Viewport {
            physical_position: match vcam.anchor {
                Anchor::TopLeft => fractional_size,
                Anchor::BottomRight => size - fractional_size,
            },
            physical_size: fractional_size,
            ..default()
        });
    }
}
