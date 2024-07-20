use bevy::{
    color::palettes,
    core_pipeline::Skybox,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
    utils::HashMap,
};
use bevy_editor_cam::{prelude::EditorCam, DefaultEditorCamPlugins};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_picking::{debug::DebugPickingMode, DefaultPickingPlugins};
use std::f32::consts::{FRAC_PI_4, PI};

fn should_remake(point: Res<ImagePoints>, planes: Res<ImagePlanes>, size: Res<ImageSize>) -> bool {
    point.is_changed() || planes.is_changed() || size.is_changed()
}

fn main() {
    App::new()
        .init_resource::<ImagePlanes>()
        .register_type::<ImagePlanes>()
        .init_resource::<ImagePoints>()
        .register_type::<ImagePoints>()
        .init_resource::<ImageSize>()
        .register_type::<ImageSize>()
        .init_resource::<MaterialsCache>()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            DefaultPickingPlugins,
            DefaultEditorCamPlugins,
        ))
        .insert_resource(DebugPickingMode::Normal)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (image_planes, (generate_points, generate_sub_points).chain()).run_if(should_remake),
        )
        .add_systems(Update, gizmo_world_axes)
        .add_systems(Update, animate_light_direction)
        .run();
}

#[derive(Debug, Resource, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
struct ImageSize(Vec2);

impl Default for ImageSize {
    fn default() -> Self {
        Self(Vec2::new(2.0, 2.0))
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::default().looking_at(Vec3::Z, Vec3::Y),
            ..default()
        },
        EditorCam::default(),
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
    ));

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

fn gizmo_world_axes(mut gizmos: Gizmos) {
    gizmos.axes(Transform::default(), 1.0);
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
        Self { num_planes: 3 }
    }
}

fn image_planes(
    mut commands: Commands,
    mut rect: Local<Option<Handle<Mesh>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mat: Local<Option<Handle<StandardMaterial>>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    size: Res<ImageSize>,
    planes: Res<ImagePlanes>,
    existing_planes: Query<Entity, With<ImagePlane>>,
) {
    for entity in &existing_planes {
        commands.entity(entity).despawn_recursive();
    }

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
        let i = i as f32;

        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh.clone_weak(),
                material: material.clone_weak(),
                transform: Transform::from_translation(Vec3::Z * i)
                    // Plane has normal +Y, so size is defined in terms of XZ.
                    .with_scale(Vec3::new(size.x * i, 0.01, size.y * i))
                    // We then also have to rotate 90 deg in X to have a plane in XY.
                    .with_rotation(Quat::from_axis_angle(Vec3::X, -std::f32::consts::FRAC_PI_2)),
                ..default()
            },
            ImagePlane,
            Name::new("plane"),
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
            num_points: 50,
            point_size: 0.01,
        }
    }
}

#[derive(Debug, Component)]
struct ImagePoint {
    index: usize,
}

/// For points in planes with Z > 1.0
#[derive(Debug, Component)]
struct SubImagePoint;

#[derive(Debug, Default, Resource)]
struct MaterialsCache {
    cache: HashMap<usize, Handle<StandardMaterial>>,
    colors: HashMap<usize, Color>,
}
impl MaterialsCache {
    fn get_or_make(
        &mut self,
        materials: &mut Assets<StandardMaterial>,
        key: usize,
    ) -> (Handle<StandardMaterial>, Color) {
        if self.cache.contains_key(&key) {
            (
                self.cache.get(&key).unwrap().clone_weak(),
                *self.colors.get(&key).unwrap(),
            )
        } else {
            let color = Color::srgb_from_array(rand::random());
            let mut smat: StandardMaterial = color.into();

            smat.unlit = true;
            smat.cull_mode = None;
            let handle = materials.add(smat);
            let weak_handle = handle.clone_weak();
            self.cache.insert(key, handle);
            self.colors.insert(key, color);
            (weak_handle, color)
        }
    }
}

fn generate_points(
    mut commands: Commands,
    mut rect: Local<Option<Handle<Mesh>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut materials_cache: ResMut<MaterialsCache>,
    size: Res<ImageSize>,
    points: Res<ImagePoints>,
    existing_points: Query<Entity, With<ImagePoint>>,
) {
    for entity in &existing_points {
        commands.entity(entity).despawn_recursive();
    }

    let mesh = rect
        .get_or_insert_with(|| meshes.add(Sphere::default().mesh()))
        .clone_weak();

    let rect = Rectangle::new(size.x, size.y);

    for index in 0..points.num_points {
        let pos = rect.sample_interior(&mut rand::thread_rng());
        let (material, _) = materials_cache.get_or_make(&mut materials, index);

        commands.spawn((
            MaterialMeshBundle {
                mesh: mesh.clone_weak(),
                material,
                transform: Transform::from_translation(pos.extend(1.0))
                    .with_scale(Vec3::splat(points.point_size)),
                ..default()
            },
            ImagePoint { index },
            Name::new("point"),
        ));
    }
}

fn generate_sub_points(
    mut commands: Commands,
    mut rect: Local<Option<Handle<Mesh>>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut materials_cache: ResMut<MaterialsCache>,
    planes: Res<ImagePlanes>,
    points: Query<(&ImagePoint, &Transform)>,
    sub_points: Query<Entity, With<SubImagePoint>>,
) {
    for entity in &sub_points {
        commands.entity(entity).despawn_recursive();
    }

    if planes.num_planes < 2 {
        return;
    }

    let mesh: Handle<Mesh> = rect
        .get_or_insert_with(|| meshes.add(Sphere::default().mesh()))
        .clone_weak();

    // let rect = Rectangle::new(size.x, size.y);

    for (image_point, transform) in &points {
        let (material, _) = materials_cache.get_or_make(&mut materials, image_point.index);

        for plane_index in 2..=planes.num_planes {
            let z = plane_index as f32;
            let translation = transform.translation * z;

            commands.spawn((
                MaterialMeshBundle {
                    mesh: mesh.clone_weak(),
                    material: material.clone_weak(),
                    transform: Transform::from_translation(translation).with_scale(transform.scale),
                    ..default()
                },
                SubImagePoint,
                Name::new("sub-point"),
            ));
        }
    }
}
