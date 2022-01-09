use bevy::prelude::*;

use crate::{MouseMotion, MouseTrackingSystem};

fn update_mouse_motion(mut events: EventReader<MouseMotion>, mut res: ResMut<MouseMotion>) {
    let delta = events.iter().fold(Vec2::ZERO, |acc, e| acc + e.delta);
    *res = MouseMotion { delta };
}

/// Plugin that tracks changes the mouse motion.
pub struct MouseMotionPlugin;

impl bevy::app::Plugin for MouseMotionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.insert_resource(MouseMotion { delta: Vec2::ZERO });
        app.add_system_to_stage(
            CoreStage::First,
            update_mouse_motion.label(MouseTrackingSystem::Motion),
        );
    }
}
