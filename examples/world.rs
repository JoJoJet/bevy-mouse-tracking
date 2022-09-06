use bevy::prelude::*;

use bevy_mouse_tracking_plugin::{prelude::*, MainCamera, MousePos, MousePosWorld};

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct Hud;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor::default())
        .add_plugin(MousePosPlugin)
        .add_startup_system(setup)
        .add_system(bevy::window::close_on_esc)
        .add_system(run)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, window: Res<WindowDescriptor>) {
    // Spawn a Camera
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.scale = 0.5; // works fine with non-unit scaling.
    commands
        .spawn_bundle(camera_bundle)
        .add_world_tracking()
        .insert(MainCamera);

    // Reference for the origin
    commands.spawn_bundle(SpriteBundle {
        texture: asset_server.load("origin.png"),
        ..Default::default()
    });

    // Reference for the mouse position
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("cursor.png"),
            ..Default::default()
        })
        .insert(Cursor);

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
    let (win_width, win_height) = (window.width, window.height);
    let (hud_x, hud_y) = (win_width / 2. * -1., win_height / 2.);
    let translation = Vec3::new(hud_x, hud_y, 0.);
    let transform = Transform::from_translation(translation);
    let value = "Screen: (-, -)\nWorld: (-, -)".to_string();

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(value, style).with_alignment(alignment),
            transform,
            ..Default::default()
        })
        .insert(Hud);
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
