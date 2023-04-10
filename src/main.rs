use ::bounded_vec_deque::BoundedVecDeque;
use bevy::{prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle};
use bevy_editor_pls::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

const BACKGROUND_COLOR: Color = Color::BLACK;
const TIME_STEP: f32 = 1. / 60.;
const GRAVITATIONAL_CONSTANT: f32 = 0.667; 

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        // .add_plugin(EditorPlugin)
        // .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        // .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugin(PanCamPlugin::default())
        .add_startup_system(setup)
        .add_systems(
            (
                apply_gravity,
                apply_acceleration.after(apply_gravity),
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

#[derive(Component, Deref, DerefMut, Debug)]
struct Acceleration(Vec2);

#[derive(Component, Debug)]
struct Mass {
    value: u64,
}

#[derive(Component, Debug)]
struct Radius {
    value: f32,
}

#[derive(Component, Deref, DerefMut)]
struct Trace(BoundedVecDeque<Vec2>);

#[derive(Component)]
struct BallTrace;

#[derive(Component)]
struct CoolDown {
    timer: Timer,
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

    // timer
    commands.spawn(CoolDown {
        timer: Timer::from_seconds(1.5, TimerMode::Once),
    });

    // camera
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(2500.0);

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
        Mass { value: 1000000 },
        Radius { value: planet_radius },
        Position(Vec2::new(0., 0.)),
    ));

    // circle
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(shape::Circle::new(15.).into()).into(),
            material: materials.add(ColorMaterial::from(Color::GREEN)),
            transform: Transform::from_xyz(0., planet_radius + 200., 1.),
            ..default()
        },
        Ball,
        Mass { value: 1},
        Radius { value: 15.},
        Velocity(Vec2::new(250., 0.)),
        Acceleration(Vec2::new(0., 0.)),
        Trace(BoundedVecDeque::new(100)),
    ));
}

fn get_distance(point_a: &Vec3, point_b: Vec3) -> f32 {
    let square_distance: f32 = (point_b.x - point_a.x).powf(2.) + (point_b.y - point_a.y).powf(2.);
    square_distance.sqrt()
}


fn apply_gravity(
    mut query_ball: Query<(&Transform, &mut Acceleration, &Mass), With<Ball>>,
    query_planet: Query<&Mass, With<Planet>>,
) {
    let planet_mass = query_planet.single();
    for (transform, mut acceleration, mass) in &mut query_ball {
        println!("{:?}, {:?}", transform.translation, mass);
        let mass_product = planet_mass.value * mass.value;
        let mut distance = get_distance(&transform.translation, Vec3::new(0.,0., 1.));
        if distance < 1. {
            distance = 1.;
        }
        let distance_product = distance * distance;
        let gravitational_force = GRAVITATIONAL_CONSTANT * mass_product as f32 / distance_product;

        let cos = -transform.translation.x / distance;
        let sen = -transform.translation.y / distance;
    

        let force_x = gravitational_force * cos;
        let force_y = gravitational_force * sen;

        println!("{} {}", force_x, force_y);

        acceleration.x = force_x / mass.value as f32;
        acceleration.y = force_y / mass.value as f32;

    }
}


fn apply_acceleration(mut query: Query<(&mut Velocity, &Acceleration)>) {
    for (mut velocity, acceleration) in &mut query {
        println!("{:?}", acceleration);
        velocity.x += acceleration.x;
        velocity.y += acceleration.y;
    }
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity, &mut Trace)>,
    mut timer: Query<&mut CoolDown>,
    time: Res<Time>,
) {
    for (mut transform, velocity, mut trace_elements) in &mut query {
        for mut clock in &mut timer {
            if clock.timer.tick(time.delta()).finished() {
                trace_elements
                    .push_back(Vec2::new(transform.translation.x, transform.translation.y));
                clock.timer = Timer::from_seconds(0.2, TimerMode::Once);
            }
        }

        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

#[allow(dead_code)]
fn clean_trace(mut commands: Commands, query: Query<Entity, With<BallTrace>>) {
    for trace_entity in &query {
        commands.entity(trace_entity).despawn();
    }
}

#[allow(dead_code)]
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
