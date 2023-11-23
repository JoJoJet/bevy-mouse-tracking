use bevy::prelude::*;

use bevy::input::mouse::MouseMotion as BevyMouseMotion;

/// Plugin that tracks mouse motion.
pub struct MouseMotionPlugin;

#[derive(Debug, Resource, Clone, Copy, PartialEq, Event)]
pub struct MouseMotion {
    pub delta: Vec2,
}

impl bevy::app::Plugin for MouseMotionPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_event::<MouseMotion>();
        app.insert_resource(MouseMotion { delta: Vec2::ZERO });
        app.add_systems(
            First,
            // update_mouse_motion.after(Events::<MouseMotion>::update_system),
            update_mouse_motion.after(bevy::ecs::event::event_update_system::<MouseMotion>),
        );
    }
}


fn update_mouse_motion(mut events: EventReader<BevyMouseMotion>, mut res: ResMut<MouseMotion>) {
    let delta = events.read().fold(Vec2::ZERO, |acc, e| acc + e.delta);
    *res = MouseMotion { delta };
}
