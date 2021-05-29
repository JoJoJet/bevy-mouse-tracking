use bevy::{
    app::{EventReader, Events},
    ecs::system::IntoSystem,
    input::mouse::MouseMotion,
    math::Vec2,
};
use bevy::ecs::system::{Local, Res, ResMut};
use bevy::app::CoreStage;

fn update_mouse_motion(
    mut events: EventReader<MouseMotion>,
    mut res: ResMut<MouseMotion>,
) {
    let delta = events
        .iter()
        .fold(Vec2::zero(), |acc, e| acc + e.delta);
    *res = MouseMotion { delta };
}

/// Plugin that tracks changes the mouse motion.
pub struct MouseMotionPlugin;

impl bevy::app::Plugin for MouseMotionPlugin {
    fn build(&self, app: &mut bevy::app::AppBuilder) {
        app.insert_resource(MouseMotion {
            delta: Vec2::zero(),
        })
        .add_system_to_stage(CoreStage::First, update_mouse_motion.system());
    }
}
