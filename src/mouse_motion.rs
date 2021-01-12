use bevy::{
    app::{stage, EventReader, Events},
    ecs::{IntoSystem as _, Local, Res, ResMut},
    input::mouse::MouseMotion,
    math::Vec2,
};

fn update_mouse_motion(
    mut event_reader: Local<EventReader<MouseMotion>>,
    events: Res<Events<MouseMotion>>,
    mut res: ResMut<MouseMotion>,
) {
    let delta = event_reader
        .iter(&events)
        .fold(Vec2::zero(), |acc, e| acc + e.delta);
    *res = MouseMotion { delta };
}

/// Plugin that tracks changes the mouse motion.
pub struct MouseMotionPlugin;

impl bevy::app::Plugin for MouseMotionPlugin {
    fn build(&self, app: &mut bevy::app::AppBuilder) {
        app.add_resource(MouseMotion {
            delta: Vec2::zero(),
        })
        .add_system_to_stage(stage::EVENT, update_mouse_motion.system());
    }
}
