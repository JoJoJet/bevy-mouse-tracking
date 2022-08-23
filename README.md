# bevy_mouse_tracking_plugin

<!-- cargo-rdme start -->

[![CI](https://github.com/JoJoJet/bevy-mouse-tracking/actions/workflows/ci.yml/badge.svg)](https://github.com/JoJoJet/bevy-mouse-tracking/workflows/ci.yml)
[![bevy_mouse_tracking on crates.io](https://img.shields.io/crates/v/bevy_mouse_tracking_plugin.svg)](https://crates.io/crates/bevy_mouse_tracking_plugin)
[![bevy_mouse_tracking docs](https://img.shields.io/badge/docs-docs.rs-orange.svg)](https://docs.rs/bevy_mouse_tracking_plugin)

Tracking the mouse in `bevy` is kind of annoying.
You gotta use [`Events`], and [`EventReader`]s, and even then, they only
get called when the mouse actually *moves*.

[`Events`]: bevy::ecs::event::Events
[`EventReader`]: bevy::ecs::event::EventReader

This crate aims to make this as easy as possible, by providing a
static [resource](bevy::ecs::system::Res) that tracks the mouse position every frame.

This crate also supports more complex use cases such as multiple cameras, which are discussed further down.

## Basics

```rust
use bevy::prelude::*;
use bevy_mouse_tracking_plugin::MousePosPlugin;

// First, add the plugin to your `App`.

App::new()
    .add_plugins(DefaultPlugins)
    .add_plugin(MousePosPlugin)
    .add_startup_system(setup)
    .add_system(dbg_mouse)
    // ...

use bevy_mouse_tracking_plugin::MousePos;

fn setup(mut commands: Commands) {
    commands
        // Spawn a camera bundle
        .spawn_bundle(Camera2dBundle::default())
        // Opt in to mouse tracking
        .insert(MousePos::default());
}

// Now, we can track the mouse position by querying for it.

fn dbg_mouse(mouse: Query<&MousePos>) {
    // This will print the screen-space location of the mouse on every frame.
    eprintln!("{}", *mouse.single());
    // If we did `mouse.iter()` instead, this will naturally work for multiple cameras.
}
```

Having to call `Query::single` is a bit annoying, and potentially error-prone.
Instead, we can specify a main camera, which the plugin will treat specially.

```rust
use bevy_mouse_tracking_plugin::MainCamera;

fn setup(mut commands: Commands) {
    let camera_id = commands
        // Spawn a camera bundle
        .spawn_bundle(Camera2dBundle::default())
        // Opt in to mouse tracking
        .insert(MousePos::default())
        // Get the ID of the camera entity we just spawned
        .id();

    // Define the `MainCamera` resource.
    commands.insert_resource(MainCamera(camera_id));
}

// Now that we've specified the main camera, we can get the mouse position using a global resource.

fn dbg_mouse(mouse: Res<MousePos>) {
    // This will print the screen-space location of the mouse on every frame.
    eprintln!("{}", *mouse);
}
```

## World-space

We can do better than just screen-space: we support automatic
transformation to world-space coordinates via [`MousePosWorld`]
-- this is can be accessed as either a component or a resource.

```rust
use bevy_mouse_tracking_plugin::MainCamera;

fn setup(mut commands: Commands) {
    let camera_id = commands
        // Spawn a camera bundle
        .spawn_bundle(Camera2dBundle::default())
        // Opt in to mouse tracking
        .insert(MousePos::default())
        .insert(MousePosWorld::default())
        // Get the ID of the camera entity we just spawned
        .id();

    // Define the `MainCamera` resource.
    commands.insert_resource(MainCamera(camera_id));
}

// Now that we've specified the main camera, we can get the mouse position using a global resource.

fn dbg_world_single(mouse: Query<&MousePosWorld>) {
    // This will print the world-space location of the mouse on every frame.
    eprintln!("{}", *mouse.single());
}

fn dbg_world_res(mouse: Res<MousePosWorld>) {
    eprintln!("{}", *mouse);
}
```

Note that this is only supported for two-dimensional, orthographic cameras,
but pull requests for 3D support are welcome!

If you do not specify a [`MainCamera`] resource, the [`MousePos`] and [`MousePosWorld`]
resources will still exist, but they will always be zero.

## Mouse motion

This crate supports a resource that tracks mouse motion, via [`MouseMotionPlugin`].
The motion can be accessed from any system in a [`MouseMotion`] resource.

[`Res`]: bevy::ecs::system::Res

<!-- cargo-rdme end -->

## Crate name

As a final aside: the name of this crate is intentionally verbose,
since it is very likely that this crate will eventually be made redundant by future updates to Bevy.  
I recommend renaming the crate in your `Cargo.toml`:
```toml
[dependencies]
mouse_tracking = { package = "bevy_mouse_tracking_plugin", version = "..." }
```

License: MIT
