use std::time::Duration;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::transform::components::Transform;
use colorgrad::Gradient;
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrInstance, XrSession};
use bevy_openxr::xr_input::oculus_touch::OculusController;
use bevy_openxr::xr_input::{Hand, QuatConv, Vec3Conv};
use bevy_openxr::DefaultXrPlugins;


fn main() {
    color_eyre::install().unwrap();

    info!("Running `openxr-6dof` skill");
    App::new()
        .add_plugins(DefaultXrPlugins)
        // .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, hands)
        .add_systems(Update, spawn_cubes)
        .insert_resource(AmbientLight::default())
        .run();
}

#[derive(Component, Default)]
pub struct CubeSpawner {
    pub current_width: i32,
    pub current_height: i32,
    pub cube_size: f32,
    pub distance: f32,
    pub increment: i32,
    pub spawned_cubes: Vec<Entity>,
    pub timer: Timer,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    let cube_size = 0.01;
    commands.spawn(CubeSpawner {
        current_width: 10,
        current_height: 10,
        cube_size: cube_size,
        distance: 0.001,
        increment: 5,
        spawned_cubes: vec![],
        timer: Timer::new(Duration::from_secs_f32(0.5), TimerMode::Repeating),
        mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size})),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    });

    // light
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 1500.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(4.0, 8.0, 4.0),
    //     ..default()
    // });


    // // camera
    // commands.spawn((Camera3dBundle {
    //     transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // }, ));
}

pub struct ControllerInput {}

fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    timer: Res<Time>,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    mut cube_spawner_query: Query<(&mut CubeSpawner)>,
) {
    let mut func = || -> color_eyre::Result<()> {
        let frame_state = *frame_state.lock().unwrap();
        let mut cube_spawner = cube_spawner_query.single_mut();
        let mut did_change = false;

        cube_spawner.timer.tick(timer.delta());

        if !cube_spawner.timer.finished() { return Ok(()); }

        let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);

        if controller.thumbstick(Hand::Right).x > 0.5 {
            did_change = true;
            cube_spawner.current_width += cube_spawner.increment;
            println!("width more: {}", cube_spawner.current_width)
        }

        if controller.thumbstick(Hand::Right).x < -0.5 {
            did_change = true;
            cube_spawner.current_width -= cube_spawner.increment;
            cube_spawner.current_width = cube_spawner.current_width.max(0);
            println!("width less: {}", cube_spawner.current_width)
        }

        if controller.thumbstick(Hand::Right).y > 0.5 {
            did_change = true;
            cube_spawner.current_height += cube_spawner.increment;
            println!("height more: {}", cube_spawner.current_height)
        }

        if controller.thumbstick(Hand::Right).y < -0.5 {
            did_change = true;
            cube_spawner.current_height -= cube_spawner.increment;
            cube_spawner.current_height = cube_spawner.current_height.max(0);
            println!("height less: {}", cube_spawner.current_height)
        }

        if !did_change { return Ok(()); }

        println!("we should be spawning stuff now {}", cube_spawner.current_width * cube_spawner.current_height);
        for cube in cube_spawner.spawned_cubes.iter() {
            commands.entity(*cube).despawn();
        }
        cube_spawner.spawned_cubes.clear();

        let half_width = (cube_spawner.current_width as f32 * 0.5) as i32;
        for x in -half_width..half_width {
            for y in 0..cube_spawner.current_height {
                let cube_entity = commands.spawn(PbrBundle {
                    mesh: cube_spawner.mesh.clone_weak(),
                    material: cube_spawner.material.clone_weak(),
                    transform: Transform::from_xyz(x as f32 * (cube_spawner.cube_size + cube_spawner.distance), y as f32 * (cube_spawner.cube_size  + cube_spawner.distance), -6.0),
                    ..default()
                }).id();
                cube_spawner.spawned_cubes.push(cube_entity);
            }
        }


        Ok(())
    };

    let _ = func();
}


fn hands(
    mut gizmos: Gizmos,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    diagnostics: Res<DiagnosticsStore>,
) {
    let mut fps = 0;
    for diagnostic in diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.is_enabled)
    {
        if diagnostic.name != "fps" { continue; };
        fps = diagnostic.average().unwrap_or(0.0) as i32;
    }

    let mut func = || -> color_eyre::Result<()> {
        let frame_state = *frame_state.lock().unwrap();
        let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);

        let right_controller = controller.grip_space(Hand::Right);
        let left_controller = controller.grip_space(Hand::Left);

        let mut color = Color::YELLOW_GREEN;

        let grad = colorgrad::GradientBuilder::new()
            .html_colors(&["deeppink", "gold", "seagreen"])
            .domain(&[0.0, 72.0])
            .build::<colorgrad::LinearGradient>()?;


        if controller.a_button() {
            color = Color::BLUE;
        }
        if controller.b_button() {
            color = Color::RED;
        }
        let fps_color = grad.at(fps as f32).to_array();
        color = Color::from(fps_color);
        if controller.trigger(Hand::Right) != 0.0 {
            color = Color::rgb(
                controller.trigger(Hand::Right),
                0.5,
                controller.trigger(Hand::Right),
            );
        }

        gizmos.rect(
            right_controller.0.pose.position.to_vec3(),
            right_controller.0.pose.orientation.to_quat(),
            Vec2::new(0.05, 0.2),
            color,
        );
        gizmos.rect(
            left_controller.0.pose.position.to_vec3(),
            left_controller.0.pose.orientation.to_quat(),
            Vec2::new(0.05, 0.2),
            color,
        );
        Ok(())
    };

    let _ = func();
}
