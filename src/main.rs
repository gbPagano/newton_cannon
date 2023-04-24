use ::bounded_vec_deque::BoundedVecDeque;
use bevy::{prelude::*, render::camera::ScalingMode, sprite::MaterialMesh2dBundle};
use bevy_pancam::{PanCam, PanCamPlugin};

const BACKGROUND_COLOR: Color = Color::BLACK;
const TIME_STEP: f32 = 1. / 60.;
const SPEED_STEP: f32 = 3.5;
const TRACE_RATE: i32 = 5;
const GRAVITATIONAL_CONSTANT: f32 = 6.67;
const RESTITUTION_CONSTANT: f32 = 0.7;
const PLANET_RADIUS: f32 = 756.8 / 2.;
const PLANET_MASS: u64 = 1_000_000;
const BALL_RADIUS: f32 = 7.5;
const BALL_MASS: u64 = 1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Newton Cannon".into(),
                // resolution: (1920., 980.).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(FixedTime::new_from_secs(TIME_STEP))
        .add_plugin(PanCamPlugin::default())
        .add_startup_system(setup)
        .add_system(next_ball_events)
        .add_systems(
            (
                apply_gravity,
                apply_acceleration.after(apply_gravity),
                apply_velocity.after(apply_acceleration),
                update_texts.after(apply_velocity),
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

#[derive(Component, Deref, DerefMut, Debug)]
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

#[derive(Component)]
struct NextSpeedText;

#[derive(Component)]
struct LastBallSpeedText;

#[derive(Component)]
struct LastBallAccelerationText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // assets
    let background_image = asset_server.load("background.jpg");
    let planet_image = asset_server.load("planet_earth.png");
    let cannon_image = asset_server.load("cannon.png");
    let tower_image = asset_server.load("tower.png");

    // camera
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(2500.0);

    commands.spawn((
        cam,
        PanCam {
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
            transform: Transform {
                translation: Vec3::new(0., 0., 1.),
                scale: Vec3::splat(0.1),
                ..default()
            },
            ..default()
        },
        Planet,
        Mass { value: PLANET_MASS },
        Radius {
            value: PLANET_RADIUS,
        },
        Position(Vec2::new(0., 0.)),
    ));

    // tower and cannon
    commands.spawn(SpriteBundle {
        texture: cannon_image,
        transform: Transform {
            translation: Vec3::new(12., 510., 4.),
            scale: Vec3::splat(0.1),
            ..default()
        },
        ..default()
    });
    commands.spawn(SpriteBundle {
        texture: tower_image,
        transform: Transform {
            translation: Vec3::new(45., 425., 5.),
            scale: Vec3::splat(0.8),
            ..default()
        },
        ..default()
    });

    // speed first ball
    commands.spawn(NextBall { speed: 35. });

    // texts
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "INITIAL SPEED OF NEXT BALL: ",
                TextStyle {
                    font: asset_server.load("fonts/BebasNeue-Regular.ttf"),
                    font_size: 30.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::from_style(TextStyle {
                font: asset_server.load("fonts/BebasNeue-Regular.ttf"),
                font_size: 30.0,
                color: Color::GOLD,
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
        NextSpeedText,
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "SPEED OF LAST BALL: ",
                TextStyle {
                    font: asset_server.load("fonts/BebasNeue-Regular.ttf"),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "(X, X)",
                TextStyle {
                    font: asset_server.load("fonts/BebasNeue-Regular.ttf"),
                    font_size: 25.0,
                    color: Color::GOLD,
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(30.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
        LastBallSpeedText,
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "ACCELERATION OF LAST BALL: ",
                TextStyle {
                    font: asset_server.load("fonts/BebasNeue-Regular.ttf"),
                    font_size: 25.0,
                    color: Color::WHITE,
                },
            ),
            TextSection::new(
                "(X, X)",
                TextStyle {
                    font: asset_server.load("fonts/BebasNeue-Regular.ttf"),
                    font_size: 25.0,
                    color: Color::GOLD,
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                left: Val::Px(5.0),
                ..default()
            },
            ..default()
        }),
        LastBallAccelerationText,
    ));
}

fn update_texts(
    mut next_ball_text_query: Query<&mut Text, With<NextSpeedText>>,
    mut speed_ball_text_query: Query<&mut Text, (With<LastBallSpeedText>, Without<NextSpeedText>)>,
    mut acc_ball_text_query: Query<
        &mut Text,
        (
            With<LastBallAccelerationText>,
            Without<LastBallSpeedText>,
            Without<NextSpeedText>,
        ),
    >,
    next_ball_query: Query<&mut NextBall>,
    query_actual_ball: Query<(&Acceleration, &Velocity, &Trace)>,
) {
    let next_ball = next_ball_query.single();
    let mut next_ball_text = next_ball_text_query.single_mut();
    let mut speed_ball_text = speed_ball_text_query.single_mut();
    let mut acceleration_ball_text = acc_ball_text_query.single_mut();

    next_ball_text.sections[1].value = format!("{}", next_ball.speed);

    for (acceleration, velocity, trace) in &query_actual_ball {
        if trace.last {
            speed_ball_text.sections[1].value = format!("({:.2}, {:.2})", velocity.x, velocity.y);
            acceleration_ball_text.sections[1].value =
                format!("({:.2}, {:.2})", acceleration.x, acceleration.y);
        }
    }
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
                transform: Transform::from_xyz(60., PLANET_RADIUS + 145., 7.),
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
        next_ball.speed += 15.;
    }
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
        let mass_product = planet_mass.value * mass.value;
        let mut distance = get_distance(&transform.translation, Vec3::new(0., 0., 1.));
        if distance < 1. {
            distance = 1.;
        }
        let distance_product = distance * distance;
        let gravitational_force = GRAVITATIONAL_CONSTANT * mass_product as f32 / distance_product;

        let cos = -transform.translation.x / distance;
        let sen = -transform.translation.y / distance;

        let force_x = gravitational_force * cos;
        let force_y = gravitational_force * sen;

        acceleration.x = force_x / mass.value as f32;
        acceleration.y = force_y / mass.value as f32;
    }
}

fn apply_acceleration(mut query: Query<(&mut Velocity, &Acceleration)>) {
    for (mut velocity, acceleration) in &mut query {
        velocity.x += acceleration.x * TIME_STEP * SPEED_STEP;
        velocity.y += acceleration.y * TIME_STEP * SPEED_STEP;
    }
}

fn apply_velocity(mut query: Query<(&mut Transform, &mut Velocity, &Radius, &mut Trace)>) {
    for (mut transform, mut velocity, radius, mut trace) in &mut query {
        if trace.last {
            trace.counter += 1;
            if trace.counter % TRACE_RATE == 0 {
                trace
                    .balls
                    .push_back(Vec2::new(transform.translation.x, transform.translation.y));
            }
        }

        transform.translation.x += velocity.x * TIME_STEP * SPEED_STEP;
        transform.translation.y += velocity.y * TIME_STEP * SPEED_STEP;

        // verify collison
        let distance = get_distance(&transform.translation, Vec3::new(0., 0., 1.));
        if distance < PLANET_RADIUS + radius.value {
            transform.translation.x -= velocity.x * TIME_STEP * SPEED_STEP;
            transform.translation.y -= velocity.y * TIME_STEP * SPEED_STEP;

            // apply colission
            velocity.x *= RESTITUTION_CONSTANT;
            velocity.y *= RESTITUTION_CONSTANT;

            let m1 = PLANET_MASS as f32;
            let m2 = BALL_MASS as f32;

            let u1x: f32 = 0.; // planet velocity
            let u1y: f32 = 0.; // planet velocity
            let u2x = velocity.x;
            let u2y = velocity.y;

            let x1 = 0.; // planet position
            let y1 = 0.; // planet position
            let x2 = transform.translation.x;
            let y2 = transform.translation.y;

            let u1 = ((u1x * u1x + u1y * u1y) as f32).sqrt();
            let u2 = ((u2x * u2x + u2y * u2y) as f32).sqrt();

            let a1 = (y2 - y1).atan2(x2 - x1);
            let b1 = u1y.atan2(u1x);
            let c1 = b1 - a1;

            let a2 = (y1 - y2).atan2(x1 - x2);
            let b2 = u2y.atan2(u2x);
            let c2 = b2 - a2;

            let u12 = u1 * c1.cos();
            let u11 = u1 * c1.sin();

            let u21 = u2 * c2.cos();
            let u22 = u2 * c2.sin();

            let v12 = (((m1 - m2) * u12) - (2. * m2 * u21)) / (m1 + m2);
            let v21 = (((m1 - m2) * u21) + (2. * m1 * u12)) / (m1 + m2);

            let v1x = u11 * -a1.sin() + v12 * a1.cos();
            let v1y = u11 * a1.cos() + v12 * a1.sin();

            let v2x = u22 * -a2.sin() - v21 * a2.cos();
            let v2y = u22 * a2.cos() - v21 * a2.sin();

            // assert planet is not moving
            assert_eq!((v1x * 100.0).round() / 100.0, 0.);
            assert_eq!((v1y * 100.0).round() / 100.0, 0.);

            // update ball velocity
            velocity.x = v2x;
            velocity.y = v2y;
        }
    }
}

fn clean_trace(mut commands: Commands, query: Query<Entity, With<BallTrace>>) {
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
    for trace in query.iter() {
        if trace.last {
            for trace_element in trace.balls.iter() {
                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes.add(shape::Circle::new(4.).into()).into(),
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
