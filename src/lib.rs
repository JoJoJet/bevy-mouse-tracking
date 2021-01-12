mod mouse_pos;
pub use mouse_pos::{MousePos, MousePosWorld, MousePosPlugin};

mod mouse_motion;
pub use mouse_motion::MouseMotionPlugin;
pub use bevy::input::mouse::MouseMotion;
