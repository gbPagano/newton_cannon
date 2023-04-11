use ::bounded_vec_deque::BoundedVecDeque;
use bevy::{prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle};
use bevy_editor_pls::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

const BACKGROUND_COLOR: Color = Color::BLACK;
const TIME_STEP: f32 = 1. / 60.;
const GRAVITATIONAL_CONSTANT: f32 = 0.35; 
const PLANET_RADIUS: f32 = 756.8 / 2.;
const PLANET_MASS: u64 = 1_000_000;
const BALL_RADIUS: f32 = 7.5;
const BALL_MASS: u64 = 1;



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
                next_ball_events.before(apply_gravity),
                apply_gravity,
                apply_acceleration.after(apply_gravity),
                apply_velocity.after(apply_acceleration),
                clean_trace.after(apply_velocity),
                spawn_trace.after(clean_trace),
            )
                // .in_schedule(CoreSchedule::FixedUpdate),
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

#[derive(Component)]
struct Trace {
    balls: BoundedVecDeque<Vec2>,
    counter: i32,
    last: bool,
}

#[derive(Component)]
struct BallTrace;



#[derive(Component)]
struct NextBall {
    speed: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // assets
    let background_image = asset_server.load("background.jpg");
    let planet_image = asset_server.load("planet_earth.png");
    let cannon_image = asset_server.load("cannon.png");




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
        Mass { value: PLANET_MASS },
        Radius { value: PLANET_RADIUS },
        Position(Vec2::new(0., 0.)),
    ));
    
    // cannon 
    commands.spawn(SpriteBundle {
            texture: cannon_image,
            // transform: Transform::from_scale(Vec3::splat(0.1)),
            transform: Transform {
                translation: Vec3::new(0., 0., 1.),
                scale: Vec3::splat(0.1),
                ..default()
            },
            ..default()
        },
    );


    // next ball
    commands.spawn(NextBall {
        speed: 0., 
    });

}


fn next_ball_events(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut NextBall>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut old_balls: Query<&mut Trace>,
) {
    let mut next_ball = query.single_mut();
    if keyboard_input.any_pressed([KeyCode::Left, KeyCode::Down]) {
        next_ball.speed -= 1.0;
    }
    if keyboard_input.any_pressed([KeyCode::Right, KeyCode::Up]) {
        next_ball.speed += 1.0;
    }
    if keyboard_input.any_just_pressed([KeyCode::Space, KeyCode::Return]) {
        
        for mut trace in &mut old_balls {
            trace.balls = BoundedVecDeque::new(1);
            trace.last = false;
        }

        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(BALL_RADIUS).into()).into(),
                material: materials.add(ColorMaterial::from(Color::RED)),
                transform: Transform::from_xyz(0., PLANET_RADIUS + 200., 2.),
                ..default()
            },
            Ball,
            Mass { value: BALL_MASS },
            Radius { value: BALL_RADIUS },
            Velocity(Vec2::new(next_ball.speed, 0.)),
            Acceleration(Vec2::new(0., 0.)),
            Trace {
                balls: BoundedVecDeque::new(100),
                counter: 0,
                last: true,
            },
        ));
        next_ball.speed += 30.;
    }
    println!("{}", next_ball.speed);
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
        // println!("{:?}, {:?}", transform.translation, mass);
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
    mut query: Query<(&mut Transform, &Velocity, &Radius, &mut Trace)>,


) {
    for (mut transform, velocity, radius, mut trace) in &mut query {
        
        
        if trace.last {
            trace.counter += 1;
            if trace.counter % 10 == 0 {
                trace.balls.push_back(Vec2::new(transform.translation.x, transform.translation.y));
            } 
        }
       

        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;

        // verificar colisao
        
        let distance = get_distance(&transform.translation, Vec3::new(0.,0., 1.));
        if distance < PLANET_RADIUS + radius.value {
            transform.translation.x -= velocity.x * TIME_STEP;
            transform.translation.y -= velocity.y * TIME_STEP;
        }

        // colisao inelastica

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
    for trace in query.iter() {
        if trace.last {
            for trace_element in trace.balls.iter() {
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

    }
}
