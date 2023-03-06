//! [![CI](https://github.com/JoJoJet/bevy-mouse-tracking/actions/workflows/ci.yml/badge.svg)](https://github.com/JoJoJet/bevy-mouse-tracking/workflows/ci.yml)
//! [![bevy_mouse_tracking on crates.io](https://img.shields.io/crates/v/bevy_mouse_tracking_plugin.svg)](https://crates.io/crates/bevy_mouse_tracking_plugin)
//! [![bevy_mouse_tracking docs](https://img.shields.io/badge/docs-docs.rs-orange.svg)](https://docs.rs/bevy_mouse_tracking_plugin)
//!
//! # Versions
//!
//! | Bevy Version | Crate Version |
//! |--------------|---------------|
//! | 0.10         | 0.6           |
//! | 0.9          | 0.5           |
//! | 0.8          | 0.4           |
//! | 0.7          | 0.2.1         |
//! | 0.6          | 0.2.0         |
//! | main branch  | main branch   |
//!
//! This crate aims to make mouse tracking both effortless and explicit.
//! Tracking is opt-in and handled opaquely by this plugin.
//!
//! The mouse can be tracked on a per-camera basis by querying for tracking components.
//! Additionally, a global resource is maintained that tracks the main camera, if applicable.
//!
//! # Basics
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_mouse_tracking_plugin::prelude::*;
//!
//! // First, add the plugin to your `App`.
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(MousePosPlugin)
//!     .add_startup_system(setup)
//!     .add_system(dbg_mouse)
//!     // ...
//! #    .update();
//!
//!
//! fn setup(mut commands: Commands) {
//!     commands
//!         // Spawn a camera bundle
//!         .spawn(Camera2dBundle::default())
//!         // Opt in to mouse tracking.
//!         // `InitMouseTracking` is a command that adds the mouse tracking
//!         // component to the camera with a correct initial value.
//!         .add(InitMouseTracking);
//! }
//!
//! // Now, we can track the mouse position by querying for it.
//!
//! use bevy_mouse_tracking_plugin::MousePos;
//!
//! fn dbg_mouse(mouse: Query<&MousePos>) {
//!     // This will print the screen-space location of the mouse on every frame.
//!     eprintln!("{}", *mouse.single());
//!     // If we did `mouse.iter()` instead, this will naturally work for multiple cameras.
//! }
//! ```
//!
//! Having to call `Query::single` is a bit annoying, and potentially error-prone.
//! Instead, we can specify a main camera, which the plugin will treat specially.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_mouse_tracking_plugin::{prelude::*, MousePos};
//! # App::new()
//! #    .add_plugins(DefaultPlugins)
//! #    .add_plugin(MousePosPlugin)
//! #    .add_startup_system(setup)
//! #    .add_system(dbg_mouse)
//! #    .update();
//! use bevy_mouse_tracking_plugin::MainCamera;
//!
//! fn setup(mut commands: Commands) {
//!     commands
//!         // Spawn a camera with tracking.
//!         .spawn(Camera2dBundle::default())
//!         .add(InitMouseTracking)
//!         // Add a component to mark it as the main camera.
//!         .insert(MainCamera);
//! }
//!
//! // Now that we've specified the main camera, we can get the mouse position using a global resource.
//!
//! fn dbg_mouse(mouse: Res<MousePos>) {
//!     // This will print the screen-space location of the mouse on every frame.
//!     eprintln!("{}", *mouse);
//! }
//! ```
//!
//! # World-space
//!
//! We can do better than just screen-space: this crate supports automatic
//! transformation to world-space coordinates via [`MousePosWorld`]
//! -- this is can be accessed as either a component or a resource.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_mouse_tracking_plugin::{prelude::*, MousePos, MainCamera};
//! # App::new()
//! #    .add_plugins(DefaultPlugins)
//! #    .add_plugin(MousePosPlugin)
//! #    .add_startup_system(setup)
//! #    .add_system(dbg_world_single)
//! #    .add_system(dbg_world_res)
//! #    .update();
//! use bevy_mouse_tracking_plugin::MousePosWorld;
//!
//! fn setup(mut commands: Commands) {
//!     commands
//!         .spawn(Camera2dBundle::default())
//!         // Opt in to world-space mouse tracking.
//!         // This will automatically opt into screen-space tracking.
//!         .add(InitWorldTracking)
//!         // ...
//! #       .insert(MainCamera);
//! }
//!
//! // Getting the world-space position using a query.
//! fn dbg_world_single(mouse: Query<&MousePosWorld>) {
//!     // This will print the world-space position of the mouse on every frame.
//!     eprintln!("{}", *mouse.single());
//! }
//!
//! // Getting it using the resource.
//! fn dbg_world_res(mouse: Res<MousePosWorld>) {
//!     eprintln!("{}", *mouse);
//! }
//! ```
//!
//! Note that this is only supported for two-dimensional, orthographic cameras,
//! but pull requests for 3D support are welcome!
//!
//! If you do not specify a [`MainCamera`], the [`MousePos`] and [`MousePosWorld`]
//! resources will still exist, but they will always be zero.
//!
//! # Mouse motion
//!
//! This crate supports a resource that tracks mouse motion, via [`MouseMotionPlugin`].
//! The motion can be accessed from any system in a [`MouseMotion`] resource.
//!
//! [`Res`]: bevy::ecs::system::Res

#![allow(clippy::type_complexity)]

pub mod prelude {
    pub use crate::mouse_motion::MouseMotionPlugin;
    pub use crate::mouse_pos::{InitMouseTracking, InitWorldTracking, MousePosPlugin};
}

pub mod mouse_pos;
pub use mouse_pos::{MainCamera, MousePos, MousePosWorld};

pub mod mouse_motion;
pub use mouse_motion::MouseMotion;
