//! [![CI](https://github.com/JoJoJet/bevy-mouse-tracking/actions/workflows/ci.yml/badge.svg)](https://github.com/JoJoJet/bevy-mouse-tracking/workflows/ci.yml)
//! [![bevy_mouse_tracking on crates.io](https://img.shields.io/crates/v/bevy_mouse_tracking_plugin.svg)](https://crates.io/crates/bevy_mouse_tracking_plugin)
//! [![bevy_mouse_tracking docs](https://img.shields.io/badge/docs-docs.rs-orange.svg)](https://docs.rs/bevy_mouse_tracking_plugin)
//!
//! Tracking the mouse in `bevy` is kind of annoying.
//! You gotta use [`Events`], and [`EventReader`]s, and even then, they only
//! get called when the mouse actually *moves*.
//!
//! [`Events`]: bevy::ecs::event::Events
//! [`EventReader`]: bevy::ecs::event::EventReader
//!
//! This crate aims to make this as easy as possible, by providing a
//! static [resource](bevy::ecs::system::Res) that tracks the mouse position every frame.
//!
//! This crate also supports more complex use cases such as multiple cameras, which are discussed further down.
//!
//! # Basics
//!
//! ```
//! use bevy::prelude::*;
//! use bevy_mouse_tracking_plugin::{MousePosPlugin, MainCamera};
//!
//! // First, add the plugin to your `App`.
//!
//! App::new()
//!     .add_plugins(DefaultPlugins)
//!     .add_plugin(MousePosPlugin)
//! #    .add_startup_system(setup)
//! #    .add_system(dbg_mouse)
//!     // ...
//!     .update();
//!
//! // Spawn a camera, and specify it as the main camera.
//!
//! fn setup(mut commands: Commands) {
//!     let camera_id = commands.spawn_bundle(Camera2dBundle::default()).id();
//!     commands.insert_resource(MainCamera(camera_id));
//! }
//!
//! // With that, you can now easily track the main camera through a global resource.
//!
//! use bevy_mouse_tracking_plugin::MousePos;
//! fn dbg_mouse(mouse: Res<MousePos>) {
//!     // This will print the screen-space location of the mouse on every frame.
//!     eprintln!("{}", *mouse);
//! }
//! ```
//!
//! We can do better than just screen-space: we support automatic
//! transformation to world-space coordinates via the [`MousePosWorld`] resource.
//!
//! ```
//! # use bevy::prelude::*;
//! use bevy_mouse_tracking_plugin::MousePosWorld;
//! fn dbg_world(mouse: Res<MousePosWorld>) {
//!     eprintln!("{}", *mouse);
//! }
//! ```
//!
//! This will print the world-space location of the mouse on every frame.  
//! Note that this is only supported for two-dimensional, orthographic cameras,
//! but pull requests for 3D support are welcome!
//!
//! Note that if you do not specify a [`MainCamera`] resource, the [`MousePos`] and [`MousePosWorld`]
//! resources will still exist, but they will always be zero.
//!
//! ## Queries
//!
//! If you want to get mouse tracking information relative to each camera individually,
//! simply [query](bevy::ecs::system::Query) for `MousePos` or `MousePosWorld` as a
//! _component_ instead of as a resource.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_mouse_tracking_plugin::{MousePosPlugin, MainCamera, MousePosWorld};
//!
//! App::new()
//!     // plugins omitted...
//! #   .add_plugins(DefaultPlugins)
//! #   .add_plugin(MousePosPlugin)
//!     .add_startup_system(setup)
//!     .add_system(dbg_for_each)
//! #    .update();
//!
//! # type MinimapCameraBundle = Camera2dBundle::default();
//! fn setup(mut commands: Commands) {
//!     // Spawn the main camera for the game...
//!     commands.spawn_bundle(Camera2dBundle::default());
//!     // ...as well as a special overhead camera for the minimap.
//!     commands.spawn_bundle(MinimapCameraBundle::default());
//! }
//!
//! fn dbg_for_each(mouse_pos: Query<&MousePosWorld>) {
//!     // This prints the mouse position twice every frame:
//!     // once relative to the main camera, and once relative to the minimap camera.
//!     # // FIXME: We should take camera-driven rendering into account somehow.
//!     for pos in mouse_pos.iter() {
//!         eprintln!("{}", *pos);
//!     }
//! }
//! ```
//!
//! ## Opt-out of tracking for cameras
//!
//! If you wish to have a camera be excluded from mouse tracking for whatever reason, you may give it the [`ExcludeMouseTracking`] component.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy_mouse_tracking_plugin::{MousePosPlugin, ExcludeMouseTracking};
//! # App::new()
//! #   .add_plugins(DefaultPlugins)
//! #   .add_plugin(MousePosPlugin)
//! #   .add_startup_system(setup)
//! #   .update();
//! # fn setup(mut commands: Commands) {
//!     commands.spawn_bundle(Camera2dBundle::default())
//!         .insert(ExcludeMouseTracking);
//! # }
//! ```
//!
//! This camera will not have a [`MousePos`] or a [`MousePosWorld`], as it is completely excluded from mouse tracking.
//!
//! One reason to do this is because this crate does not currently support cameras with projections other than Bevy's [`OrthographicProjection`](bevy::render::camera::OrthographicProjection). If you use such a camera, even if you don't use it for tracking mouse position, you will find that it panics:
//!
//! ```text
//! thread 'main' panicked at 'only orthographic cameras are supported -- consider adding an ExcludeMouseTracking component: QueryDoesNotMatch(5v0)', src\mouse_pos.rs:159:50
//! ```
//!
//! To get around this, you may choose to have the camera opt-out.
//!
//! ```
//! # use bevy::prelude::*;
//! # use bevy::render::camera::{PerspectiveProjection, Projection};
//! # use bevy_mouse_tracking_plugin::{MousePosPlugin, ExcludeMouseTracking};
//! # App::new()
//! #   .add_plugins(DefaultPlugins)
//! #   .add_plugin(MousePosPlugin)
//! #   .add_startup_system(setup)
//! #   .update();
//! # fn setup(mut commands: Commands) {
//!     commands.spawn_bundle(Camera3dBundle {
//!         projection: Projection::from(PerspectiveProjection::default()),
//!         ..default()
//!     }).insert(ExcludeMouseTracking);
//! # }
//! ```
//!
//! # Mouse motion
//!
//! This crate supports a resource that tracks mouse motion, via [`MouseMotionPlugin`].
//! The motion can be accessed from any system in a [`MouseMotion`] resource.
//!
//! [`Res`]: bevy::ecs::system::Res
//!
//! # Crate name
//!
//! As a final aside: the name of this crate is intentionally verbose.
//! This is because I didn't want to steal a crate name, especially since
//! it is very likely that this crate will eventually be made redundant by
//! future updates to `bevy`.  
//! I recommend renaming the crate in your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! mouse_tracking = { package = "bevy_mouse_tracking_plugin", version = "..." }
//! ```

mod mouse_pos;
pub use mouse_pos::{ExcludeMouseTracking, MainCamera, MousePos, MousePosPlugin, MousePosWorld};

mod mouse_motion;
pub use mouse_motion::{MouseMotion, MouseMotionPlugin};
