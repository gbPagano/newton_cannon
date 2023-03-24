use bevy::{prelude::*, sprite::MaterialMesh2dBundle, render::camera::ScalingMode};
use bevy_editor_pls::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin)
        .add_plugin(PanCamPlugin::default())
        .add_startup_system(setup)
        .run();
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Acceleration(Vec2);

#[derive(Component)]
struct Mass(u64);


fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let background_image = asset_server.load("background_2.jpg");
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(2500.0);

    commands.spawn((cam, PanCam{
            // max_scale: Some(4.),
            min_scale: 0.3,
            min_x: Some(-6016./2.0),
            max_x: Some(6016./2.0),
            min_y: Some(-4016./2.0),
            max_y: Some(4016./2.0),
            ..default()
        }));

    commands.spawn(SpriteBundle {
        texture: background_image,
        ..default()
    });
    // Circle
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(shape::Circle::new(300.).into()).into(),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
        ..default()
    });
}
