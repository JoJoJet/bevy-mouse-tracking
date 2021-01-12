use std::{fmt::Display, ops::Deref};

use bevy::{
    app::{stage, AppBuilder, EventReader, Events, Plugin},
    ecs::{IntoSystem as _, Local, Query, Res, ResMut, With},
    math::{Vec2, Vec3},
    render::camera::{Camera, OrthographicProjection},
    transform::components::GlobalTransform,
    window::CursorMoved,
};

/// The location of the mouse in screenspace.
#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub struct MousePos(pub Vec2);

impl Deref for MousePos {
    type Target = Vec2;
    fn deref(&self) -> &Vec2 {
        &self.0
    }
}

impl Display for MousePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

fn update_pos(
    mut mouse_loc: ResMut<MousePos>,
    mut event_reader: Local<EventReader<CursorMoved>>,
    cursor_moved: Res<Events<CursorMoved>>,
) {
    for event in event_reader.iter(&cursor_moved) {
        mouse_loc.0 = event.position;
    }
}

/// The location of the mouse in worldspace.
#[derive(Clone, Copy, PartialEq, PartialOrd, Default, Debug)]
pub struct MousePosWorld(pub Vec3);

impl Deref for MousePosWorld {
    type Target = Vec3;
    fn deref(&self) -> &Vec3 {
        &self.0
    }
}

fn update_pos_ortho(
    mut mouse_world: ResMut<MousePosWorld>,
    mut event_reader: Local<EventReader<CursorMoved>>,
    cursor_moved: Res<Events<CursorMoved>>,
    cameras: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
) {
    if let Some(event) = event_reader.latest(&cursor_moved) {
        let (camera, proj) = cameras
            .iter()
            .next()
            .expect("could not find an orthographic camera");
        mouse_world.0 = event.position.extend(0.0)
            + Vec3::new(proj.left, proj.bottom, proj.near)
            + camera.translation;
    }
}

/// Plugin that tracks the mouse location.
pub enum MousePosPlugin {
    /// Track the mouse without transforming it to worldspace.
    None,
    /// Transform the mouse position into worldspace, using an orthographic camera.
    Orthographic,
}

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_resource(MousePos::default())
            .add_system(update_pos.system());
        //
        // Optionally add features for converting to worldspace.
        match *self {
            MousePosPlugin::None => {}
            MousePosPlugin::Orthographic => {
                app.add_resource(MousePosWorld::default())
                    .add_system_to_stage(stage::EVENT, update_pos_ortho.system());
            }
        }
    }
}
