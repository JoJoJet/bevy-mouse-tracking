//! Tracking the mouse in `bevy` is kind of annoying.
//! You gotta use [`Events`], and [`EventReader`]s, and even then, they only
//! get called when the mouse actually *moves*.
//!
//! [`Events`]: bevy::app::Events
//! [`EventReader`]: bevy::app::EventReader
//!
//! This crate aims to make this as easy as possible, by providing a
//! static resource that tracks the mouse position every frame.  
//! First, add the plugin to your app:
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_mouse_tracking_plugin::MousePosPlugin;
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(MousePosPlugin::None);
//! }
//! ```
//!
//! Now, you can access the resource in your [`System`]s:
//!
//! [`System`]: bevy::ecs::System
//!
//! ```
//! # use bevy::prelude::*;
//! use bevy_mouse_tracking_plugin::MousePos;
//! fn dbg_mouse(mouse: Res<MousePos>) {
//!     eprintln!("{}", *mouse);
//! }
//! ```
//! ...and don't forget to add the system to your app:
//! ```
//! # use bevy::prelude::*;
//! # use bevy_mouse_tracking_plugin::MousePosPlugin;
//! # fn main() {
//! #    App::new()
//! #        .add_plugins(DefaultPlugins)
//!         .add_plugin(MousePosPlugin::None)
//!         .add_system(dbg_mouse.system());
//! # }
//! # fn dbg_mouse() { }
//! ```
//!
//! This will print the screen-space location of the mouse on every frame.
//!
//! However, we can do better than just screen-space: we support automatic
//! transformation to world-space coordinates.  
//! Change the plugin to this:
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_mouse_tracking_plugin::MousePosPlugin;
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(MousePosPlugin::Orthographic);
//! }
//! ```
//!
//! In a system...
//! ```
//! # use bevy::prelude::*;
//! use bevy_mouse_tracking_plugin::MousePosWorld;
//! fn dbg_world(mouse: Res<MousePosWorld>) {
//!     eprintln!("{}", *mouse);
//!     // Note: the screen-space position is still accessible
//! }
//! ```
//!
//! This will print the world-space location of the mouse on every frame.  
//! Note that this is only supported for two-dimensional, orthographic camera,
//! but pull requests for 3D support are welcome!
//!
//! Additionally, we also support a resource that tracks mouse motion, via [`MouseMotionPlugin`].
//! The motion can be accessed from any system in a [`MouseMotion`] [`Res`].
//!
//! [`Res`]: bevy::ecs::Res
//!
//! As a final aside: the name of this crate is intentionally verbose.
//! This is because I don't want to steal a crate name, especially since
//! it is very likely that this crate will eventually be made redundant by
//! future updates to `bevy`.  
//! I recommend renaming the crate in your `Cargo.toml`:
//! ```text
//! [dependencies]
//! mouse_tracking = { package = "bevy_mouse_tracking_plugin", version = "..." }
//! ```

mod mouse_pos;
pub use mouse_pos::{MousePos, MousePosPlugin, MousePosWorld};

mod mouse_motion;
pub use bevy::input::mouse::MouseMotion;
pub use mouse_motion::MouseMotionPlugin;
