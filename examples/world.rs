use bevy::prelude::*;

use bevy_mouse_tracking_plugin::{
    mouse_pos::InitWorldTracking, prelude::*, MainCamera, MousePos, MousePosWorld,
};

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct Hud;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(MousePosPlugin)
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .add_system(pan_camera)
        .add_system(run)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    // Spawn a Camera
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.5; // works fine with non-unit scaling.
    commands
        .spawn((camera_bundle, MainCamera))
        .add(InitWorldTracking);

    // Reference for the origin
    commands.spawn(SpriteBundle {
        texture: asset_server.load("origin.png"),
        ..Default::default()
    });

    // Reference for the mouse position
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("cursor.png"),
            ..Default::default()
        },
        Cursor,
    ));

    // Hud
    let font = asset_server.load("FiraMono-Medium.ttf");
    let style = TextStyle {
        font,
        font_size: 24.0,
        color: Color::ORANGE,
    };
    let alignment = TextAlignment {
        vertical: VerticalAlign::Top,
        horizontal: HorizontalAlign::Left,
    };
    let (win_width, win_height) = (window.width(), window.height());
    let (hud_x, hud_y) = (win_width / 2. * -1., win_height / 2.);
    let translation = Vec3::new(hud_x, hud_y, 0.);
    let transform = Transform::from_translation(translation);
    let value = "Screen: (-, -)\nWorld: (-, -)".to_string();

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(value, style).with_alignment(alignment),
            transform,
            ..Default::default()
        },
        Hud,
    ));
}

fn pan_camera(mut camera: Query<&mut Transform, With<Camera>>, input: Res<Input<KeyCode>>) {
    #[allow(clippy::obfuscated_if_else)]
    fn axis(min: KeyCode, max: KeyCode, input: &Input<KeyCode>) -> f32 {
        input.pressed(min).then_some(-1.0).unwrap_or(0.0)
            + input.pressed(max).then_some(1.0).unwrap_or(0.0)
    }
    let translation = Vec2::new(
        axis(KeyCode::Left, KeyCode::Right, &input),
        axis(KeyCode::Down, KeyCode::Up, &input),
    );

    if translation != Vec2::ZERO {
        let mut camera = camera.single_mut();
        camera.translation += translation.extend(0.0) * 5.0;
    }
}

fn run(
    mouse_screen_pos: Res<MousePos>,
    mouse_world_pos: Res<MousePosWorld>,
    mut hud_text: Query<&mut Text, With<Hud>>,
    mut cursor: Query<&mut Transform, With<Cursor>>,
) {
    let hud_value = format!(
        "Screen: ({}, {})\nWorld: ({}, {})",
        mouse_screen_pos.x, mouse_screen_pos.y, mouse_world_pos.x, mouse_world_pos.y,
    );

    if let Some(mut hud_text) = hud_text.iter_mut().next() {
        hud_text.sections.first_mut().unwrap().value = hud_value;
    }

    if let Some(mut cursor_transform) = cursor.iter_mut().next() {
        cursor_transform.translation = Vec3::new(mouse_world_pos.x, mouse_world_pos.y, 0.);
    }
}
