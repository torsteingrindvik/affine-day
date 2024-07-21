use bevy::{
    color::palettes,
    core_pipeline::Skybox,
    ecs::system::SystemParam,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::{
        texture::{ImageLoaderSettings, ImageSampler},
        view::RenderLayers,
    },
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use bevy_mod_picking::{debug::DebugPickingMode, DefaultPickingPlugins};
use egui_suppress::EguiSupressPlugin;
use gizmos::GizmosPlugin;
use material_mesh_cache::{MaterialMeshCachePlugin, MeshMaterialCache};
use std::f32::consts::{FRAC_PI_4, PI};
use transform_gizmo_bevy::{GizmoCamera, GizmoTarget, TransformGizmoPlugin};
use ui_settings::UiSettingsPlugin;
use viewport_camera::ViewportCameraPlugin;

fn should_remake(point: Res<ImagePoints>, planes: Res<ImagePlanes>, size: Res<ImageSize>) -> bool {
    point.is_changed() || planes.is_changed() || size.is_changed()
}

// Potentially re-usable stuff
pub mod egui_suppress;
pub mod gizmos;
pub mod material_mesh_cache;
pub mod viewport_camera;

// Very this-project specific stuff
pub mod ui_settings;

const MISC_LAYER: usize = 1;

fn main() {
    App::new()
        .init_resource::<ImagePlanes>()
        .register_type::<ImagePlanes>()
        .init_resource::<ImagePoints>()
        .register_type::<ImagePoints>()
        .init_resource::<ImageSize>()
        .register_type::<ImageSize>()
        .register_type::<ImagePointIndex>()
        .add_plugins((
            DefaultPlugins,
            DefaultPickingPlugins,
            DefaultEditorCamPlugins,
            TransformGizmoPlugin,
        ))
        .add_plugins((
            MaterialMeshCachePlugin,
            ViewportCameraPlugin,
            GizmosPlugin,
            UiSettingsPlugin,
            EguiSupressPlugin,
        ))
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, (setup_parent_spatial, setup).chain())
        .add_systems(
            Update,
            (clear, image_planes, generate_points, generate_sub_points)
                .chain()
                .run_if(should_remake),
        )
        .add_systems(
            Update,
            (animate_light_direction, propagate_follower_transforms),
        )
        .run();
}

fn clear(mut commands: Commands, parent: Res<MainPointsParent>) {
    commands.entity(**parent).despawn_descendants();
}

#[derive(Debug, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
struct ImageSize(Vec2);

impl Default for ImageSize {
    fn default() -> Self {
        Self(Vec2::new(1.36, 0.765))
    }
}

#[derive(Debug, Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    main_points_entity: Res<MainPointsParent>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::default().looking_at(Vec3::Z, Vec3::Y),
            camera: Camera {
                order: 0,
                ..default()
            },
            ..default()
        },
        EditorCam::default(),
        RenderLayers::default().with(MISC_LAYER),
        GizmoCamera,
        Skybox {
            image: asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                format!("skyboxes/circus_arena_4k_diffuse.ktx2"),
                |settings| {
                    settings.sampler = ImageSampler::linear();
                },
            ),
            brightness: 1000.0,
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
        },
        MainCamera,
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::default().looking_at(Vec3::Z, Vec3::Y),
            camera: Camera {
                clear_color: ClearColorConfig::None,
                order: 1,
                ..default()
            },
            ..default()
        },
        Skybox {
            image: asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                format!("skyboxes/circus_arena_4k_diffuse.ktx2"),
                |settings| {
                    settings.sampler = ImageSampler::linear();
                },
            ),
            brightness: 1000.0,
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
        },
        viewport_camera::ViewportCamera {
            anchor: viewport_camera::Anchor::BottomRight,
            fraction: Vec2::new(0.3, 0.3),
        },
        // For some reason if we spawn this camera as a child we get issues, so let's do this instead
        CopyTransformOf {
            target: **main_points_entity,
            translation: true,
            scale: true,
            rotation: true,
        },
    ));
}

#[derive(Debug, Component)]
struct CopyTransformOf {
    target: Entity,
    translation: bool,
    scale: bool,
    rotation: bool,
}

fn propagate_follower_transforms(
    mut followers: Query<(&mut Transform, &CopyTransformOf)>,
    targets: Query<&Transform, Without<CopyTransformOf>>,
) {
    for (
        mut transform,
        CopyTransformOf {
            target,
            translation,
            scale,
            rotation,
        },
    ) in &mut followers
    {
        let Ok(target) = targets.get(*target) else {
            warn!("unexpected target missing");
            continue;
        };

        if *translation {
            transform.translation = target.translation;
        }
        if *scale {
            transform.scale = target.scale;
        }
        if *rotation {
            // transform.rotation = target.rotation;
        }
    }
}

#[derive(Debug, Resource, Deref)]
struct MainPointsParent {
    entity: Entity,
}

fn setup_parent_spatial(mut commands: Commands) {
    let id = commands
        .spawn((
            SpatialBundle::INHERITED_IDENTITY,
            Name::new(format!("main points parent")),
            GizmoTarget::default(),
        ))
        .id();
    commands.insert_resource(MainPointsParent { entity: id });
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
struct ImagePlanes {
    num_planes: usize,
}

#[derive(Debug, Component)]
struct ImagePlane;

impl Default for ImagePlanes {
    fn default() -> Self {
        Self { num_planes: 7 }
    }
}

#[derive(SystemParam)]
struct MainPointsCommands<'w, 's> {
    commands: Commands<'w, 's>,
    parent: Res<'w, MainPointsParent>,
}

impl<'w, 's> MainPointsCommands<'w, 's> {
    fn spawn(&mut self, bundle: impl Bundle) {
        self.commands
            .entity(**self.parent)
            .with_children(|child_builder| {
                child_builder.spawn(bundle);
            });
    }
}

fn image_planes(
    mut commands: MainPointsCommands,
    mut rect: Local<Option<Handle<Mesh>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mat: Local<Option<Handle<StandardMaterial>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    size: Res<ImageSize>,
    planes: Res<ImagePlanes>,
) {
    let mesh = rect
        .get_or_insert_with(|| meshes.add(Plane3d::default().mesh()))
        .clone_weak();
    let material = mat
        .get_or_insert_with(|| {
            let mut smat: StandardMaterial =
                Color::from(palettes::tailwind::GREEN_300.with_alpha(0.05)).into();

            smat.unlit = true;
            smat.cull_mode = None;
            materials.add(smat)
        })
        .clone_weak();

    for i in 1..=planes.num_planes {
        let i_f32 = i as f32;

        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh.clone_weak(),
                material: material.clone_weak(),
                transform: Transform::from_translation(Vec3::Z * i_f32)
                    // Plane has normal +Y, so size is defined in terms of XZ.
                    .with_scale(Vec3::new(size.x * i_f32, 0.01, size.y * i_f32))
                    // We then also have to rotate 90 deg in X to have a plane in XY.
                    .with_rotation(Quat::from_axis_angle(Vec3::X, -std::f32::consts::FRAC_PI_2)),
                ..default()
            },
            ImagePlane,
            Name::new(format!("plane-{i}")),
        ));
    }
}

#[derive(Debug, Resource, Reflect)]
#[reflect(Resource)]
struct ImagePoints {
    num_points: usize,
    point_size: f32,
}

impl Default for ImagePoints {
    fn default() -> Self {
        Self {
            num_points: 10,
            point_size: 0.05,
        }
    }
}

#[derive(Debug, Component)]
struct ImagePoint;

#[derive(Debug, Clone, Copy, Component, Reflect)]
struct ImagePointIndex {
    index: usize,
}

/// For points in planes with Z > 1.0
#[derive(Debug, Component)]
struct SubImagePoint;

fn generate_points(
    mut commands: MainPointsCommands,
    mut cache: MeshMaterialCache,
    size: Res<ImageSize>,
    points: Res<ImagePoints>,
) {
    let rect = Rectangle::new(size.x, size.y);

    for index in 0..points.num_points {
        let pos = rect.sample_interior(&mut rand::thread_rng());

        commands.spawn((
            MaterialMeshBundle {
                mesh: cache.mesh::<Sphere>(),
                material: cache.material(index),
                transform: Transform::from_translation(pos.extend(1.0))
                    .with_scale(Vec3::splat(points.point_size)),
                ..default()
            },
            ImagePoint,
            ImagePointIndex { index },
            Name::new(format!("point-{index}")),
        ));
    }
}

fn generate_sub_points(
    mut commands: MainPointsCommands,
    mut cache: MeshMaterialCache,
    planes: Res<ImagePlanes>,
    points: Query<(&ImagePointIndex, &Transform), With<ImagePoint>>,
) {
    if planes.num_planes < 2 {
        return;
    }
    for (image_point_index, transform) in &points {
        for plane_index in 2..=planes.num_planes {
            let z = plane_index as f32;
            let translation = transform.translation * z;

            commands.spawn((
                MaterialMeshBundle {
                    mesh: cache.mesh::<Sphere>(),
                    material: cache.material(image_point_index.index),
                    transform: Transform::from_translation(translation).with_scale(transform.scale),
                    ..default()
                },
                SubImagePoint,
                *image_point_index,
                Name::new(format!(
                    "sub-point {plane_index}-{}",
                    image_point_index.index
                )),
            ));
        }
    }
}
