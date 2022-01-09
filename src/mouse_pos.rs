use std::{fmt::Display, ops::Deref};

use bevy::prelude::*;

use crate::MouseTrackingSystem;

/// The location of the mouse in screenspace.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
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

fn update_pos(mut mouse_loc: ResMut<MousePos>, mut cursor_moved: EventReader<CursorMoved>) {
    for event in cursor_moved.iter() {
        mouse_loc.0 = event.position;
    }
}

/// The location of the mouse in worldspace.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct MousePosWorld(pub Vec3);

impl Display for MousePosWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for MousePosWorld {
    type Target = Vec3;
    fn deref(&self) -> &Vec3 {
        &self.0
    }
}

fn update_pos_ortho(
    mut mouse_world: ResMut<MousePosWorld>,
    mut cursor_moved: EventReader<CursorMoved>,
    cameras: Query<(&GlobalTransform, &OrthographicProjection), With<Camera>>,
) {
    if let Some(event) = cursor_moved.iter().next_back() {
        let (camera, proj) = cameras
            .iter()
            .next()
            .expect("could not find an orthographic camera");
        mouse_world.0 = camera
            .mul_vec3(event.position.extend(0.0) + Vec3::new(proj.left, proj.bottom, proj.near));
    }
}

/// Plugin that tracks the mouse location.
#[non_exhaustive]
pub enum MousePosPlugin {
    /// Track the mouse without transforming it to worldspace.
    None,
    /// Transform the mouse position into worldspace, using an orthographic camera.
    Orthographic,
}

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MousePos::default());
        app.add_system_to_stage(
            CoreStage::First,
            update_pos.label(MouseTrackingSystem::ScreenPos),
        );
        //
        // Optionally add features for converting to worldspace.
        match *self {
            MousePosPlugin::None => {}
            MousePosPlugin::Orthographic => {
                app.insert_resource(MousePosWorld::default());
                app.add_system_to_stage(
                    CoreStage::First,
                    update_pos_ortho.label(MouseTrackingSystem::Motion),
                );
            }
        }
    }
}
