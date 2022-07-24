use bevy::prelude::*;

use bevy_mouse_tracking_plugin::{MousePos, MousePosPlugin};

#[derive(Component)]
struct Cursor;

#[derive(Component)]
struct Hud;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(WindowDescriptor::default())
        .add_plugin(MousePosPlugin::SingleCamera)
        .add_startup_system(setup)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_system(run)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, window: Res<WindowDescriptor>) {
    // Spawn a Camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Reference for the origin
    commands.spawn_bundle(SpriteBundle {
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
    let alignment = TextAlignment {
        vertical: VerticalAlign::Top,
        horizontal: HorizontalAlign::Left,
    };
    let (win_width, win_height) = (window.width, window.height);
    let (hud_x, hud_y) = (win_width / 2. * -1., win_height / 2.);
    let translation = Vec3::new(hud_x, hud_y, 0.);
    let transform = Transform::from_translation(translation);
    let value = "Mouse: (-, -)".to_string();

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::from_section(value, style.clone()).with_alignment(alignment),
            transform,
            ..Default::default()
        })
        .insert(Hud);
}

fn run(mouse_pos: Res<MousePos>, mut hud_text: Query<&mut Text, With<Hud>>) {
    let hud_value = format!("Mouse: ({}, {})", mouse_pos.x, mouse_pos.y,);

    if let Some(mut hud_text) = hud_text.iter_mut().next() {
        hud_text.sections.first_mut().unwrap().value = hud_value.clone();
    } else {
        println!("No Hud Found");
    }
}
