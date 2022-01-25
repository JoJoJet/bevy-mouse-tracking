use std::{fmt::Display, ops::Deref};

use bevy::prelude::*;

use crate::MouseTrackingSystem;

/// Plugin that tracks the mouse location.
pub struct MousePosPlugin;

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        // system to add mouse tracking components.
        // runs once after startup, and then once at the end of each frame.
        app.add_startup_system_to_stage(StartupStage::PostStartup, add_pos_components);
        app.add_system_to_stage(CoreStage::Last, add_pos_components);

        app.add_system_to_stage(
            CoreStage::First,
            update_pos.label(MouseTrackingSystem::ScreenPos),
        );
        app.add_system_to_stage(
            CoreStage::First,
            update_pos_ortho
                .label(MouseTrackingSystem::WorldPos)
                .after(MouseTrackingSystem::ScreenPos),
        );
    }
}

/// The location of the mouse in screenspace.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct MousePos(Vec2);

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

fn add_pos_components(
    cameras1: Query<(Entity, &Camera), Without<MousePos>>,
    cameras2: Query<Entity, (With<Camera>, Without<MousePosWorld>)>,
    windows: Res<Windows>,
    mut commands: Commands,
) {
    for (e, camera) in cameras1.iter() {
        // get the initial position of the cursor.
        let position = windows
            .get(camera.window)
            .and_then(|w| w.cursor_position())
            .unwrap_or_default();
        commands.entity(e).insert(MousePos(position));
    }
    for cam in cameras2.iter() {
        commands
            .entity(cam)
            .insert(MousePosWorld(Default::default()));
    }
}

fn update_pos(
    mut movement: EventReader<CursorMoved>,
    mut cameras: Query<(&Camera, &mut MousePos)>,
) {
    for &CursorMoved { id, position } in movement.iter() {
        // find all cameras corresponding to the window on which the cursor moved.
        for (_, mut pos) in cameras.iter_mut().filter(|(c, ..)| c.window == id) {
            pos.0 = position;
        }
    }
}

/// The location of the mouse in worldspace.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
pub struct MousePosWorld(Vec3);

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
    mut tracking: Query<(Entity, &mut MousePosWorld, &MousePos), Changed<MousePos>>,
    cameras: Query<(&GlobalTransform, &OrthographicProjection)>,
) {
    for (camera, mut world, screen) in tracking.iter_mut() {
        let (camera, proj) = cameras
            .get(camera)
            .expect("only orthographic cameras are supported");
        world.0 =
            camera.mul_vec3(screen.0.extend(0.0) + Vec3::new(proj.left, proj.bottom, proj.near));
    }
}
