use bevy::{prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle};
use bevy_editor_pls::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};
use ::bounded_vec_deque::BoundedVecDeque;

const BACKGROUND_COLOR: Color = Color::BLACK;
const TIME_STEP: f32 = 1. / 60.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_plugin(EditorPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugin(PanCamPlugin::default())
        .add_startup_system(setup)
        .add_systems(
            (
                apply_acceleration,
                apply_velocity.after(apply_acceleration),
                clean_trace.after(apply_velocity),
                spawn_trace.after(clean_trace),
            )
                .in_schedule(CoreSchedule::FixedUpdate),
        )
        .run();
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Position(Vec2);

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component, Deref, DerefMut)]
struct Acceleration(Vec2);

#[derive(Component)]
struct Mass(u64);

#[derive(Component)]
struct Radius(f32);

#[derive(Component, Deref, DerefMut)]
struct Trace(BoundedVecDeque<Vec2>);

#[derive(Component)]
struct BallTrace;


#[derive(Component)]
struct CoolDown {
    cd: Timer,
}



fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // assets
    let background_image = asset_server.load("background.jpg");
    let planet_image = asset_server.load("planet_earth.png");

    // camera
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(2500.0);
    
    commands.spawn(CoolDown{ cd: Timer::from_seconds(1.5, TimerMode::Once)} );

    commands.spawn((
        cam,
        PanCam {
            // max_scale: Some(4.),
            min_scale: 0.3,
            min_x: Some(-6016. / 2.0),
            max_x: Some(6016. / 2.0),
            min_y: Some(-4016. / 2.0),
            max_y: Some(4016. / 2.0),
            ..default()
        },
    ));
    



    // background
    commands.spawn(SpriteBundle {
        texture: background_image,
        ..default()
    });

    // planet
    let planet_radius = 756.8 / 2.;
    commands.spawn((
        SpriteBundle {
            texture: planet_image,
            // transform: Transform::from_scale(Vec3::splat(0.1)),
            transform: Transform {
                translation: Vec3::new(0., 0., 1.),
                scale: Vec3::splat(0.1),
                ..default()
            },
            ..default()
        },
        Planet,
        Mass(34),
        Radius(planet_radius),
        Position(Vec2::new(0., 0.)),
    ));

    // circle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(15.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::GREEN)),
            transform: Transform::from_xyz(planet_radius, planet_radius, 1.),
            ..default()
        },
        Ball,
        Velocity(Vec2::new(50., 0.)),
        Acceleration(Vec2::new(0., 5.)),
        Trace(BoundedVecDeque::new(100)),
    ));
}


fn apply_acceleration(mut query: Query<(&mut Velocity, &Acceleration)>) {
    for (mut velocity, acceleration) in &mut query {
        velocity.x += acceleration.x * TIME_STEP;
        velocity.y += acceleration.y * TIME_STEP;
    }
}


fn apply_velocity(mut commands: Commands, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Transform, &Velocity, &mut Trace)>,
    mut timer: Query<&mut CoolDown>,
    time: Res<Time>,
) {
    for (mut transform, velocity, mut trace_elements) in &mut query {

        // commands.spawn((
        //     MaterialMesh2dBundle {
        //         mesh: meshes.add(shape::Circle::new(15.).into()).into(),
        //         material: materials.add(ColorMaterial::from(Color::GREEN)),
        //         transform: Transform::from_xyz(transform.translation.x, transform.translation.y, 1.),
        //         ..default()
        //     },
        //     BallTrace,
        // ));

        for mut clock in &mut timer {
            if clock.cd.tick(time.delta()).finished() {
                trace_elements.push_back(Vec2::new(transform.translation.x, transform.translation.y));
                clock.cd = Timer::from_seconds(0.2, TimerMode::Once);

            }
        }
        // trace_elements.push_back(Vec2::new(transform.translation.x, transform.translation.y));
        
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn clean_trace(
    mut commands: Commands,
    query: Query<Entity,  With<BallTrace>>
) {
    for trace_entity in &query {
        commands.entity(trace_entity).despawn();
    }
}

fn spawn_trace(
    mut commands: Commands,
    query: Query<&Trace>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let elements = query.single();
    for trace_element in elements.iter() {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(5.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::GREEN)),
                transform: Transform::from_xyz(trace_element.x, trace_element.y, 1.),
                ..default()
            },
            BallTrace,
        ));
    }
}
