use bevy::{prelude::*, window::PrimaryWindow};

use bevy_mouse_tracking_plugin::{
    mouse_pos::InitMouseTracking, prelude::*, MainCamera, MouseMotion, MousePos,
};

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct Hud;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(MousePosPlugin)
        .add_plugins(MouseMotionPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .add_systems(Update, run)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single();

    // Spawn a Camera
    commands
        .spawn(Camera2dBundle::default())
        .add(InitMouseTracking)
        .insert(MainCamera);

    // Reference for the origin
    commands.spawn(SpriteBundle {
        texture: asset_server.load("origin.png"),
        ..Default::default()
    });

    // Hud
    let font = asset_server.load("FiraMono-Medium.ttf");
    let style = TextStyle {
        font,
        font_size: 24.0,
        color: Color::ORANGE,
    };
    let (win_width, win_height) = (window.width(), window.height());
    let (hud_x, hud_y) = (win_width / 2. * -1., win_height / 2.);
    let translation = Vec3::new(hud_x, hud_y, 0.);
    let transform = Transform::from_translation(translation);
    let value = "Mouse: (-, -)".to_string();

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(value, style).with_alignment(TextAlignment::Left),
            transform,
            ..Default::default()
        },
        Hud,
    ));
}

fn run(
    mouse_pos: Res<MousePos>,
    mouse_motion: Res<MouseMotion>,
    mut hud_text: Query<&mut Text, With<Hud>>,
) {
    let hud_value = format!(
        "Mouse: ({}, {})\nDelta: ({}, {})",
        mouse_pos.x, mouse_pos.y, mouse_motion.delta.x, mouse_motion.delta.y,
    );

    if let Some(mut hud_text) = hud_text.iter_mut().next() {
        hud_text.sections.first_mut().unwrap().value = hud_value;
    } else {
        println!("No Hud Found");
    }
}
