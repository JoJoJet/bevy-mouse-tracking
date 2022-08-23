use std::{fmt::Display, ops::Deref};

use bevy::{prelude::*, render::camera::RenderTarget};

/// Plugin that tracks the mouse location.
pub struct MousePosPlugin;

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        // System to add mouse tracking components.
        // Runs once at the end of each frame. This means that no cameras will have
        // mouse tracking components until after the first frame.
        // This might cause some issues, but it's probably for the best since,
        // during the first frame, nothing has been rendered yet.
        app.add_system_to_stage(CoreStage::PostUpdate, add_pos_components);

        app.add_system_to_stage(CoreStage::First, update_pos);
        app.add_system_to_stage(CoreStage::First, update_pos_ortho.after(update_pos));

        app.init_resource::<MousePos>();
        app.init_resource::<MousePosWorld>();
        app.add_system_to_stage(CoreStage::First, update_resources.after(update_pos_ortho));
    }
}

/// Marker component for cameras that should be excluded from mouse tracking.
/// Any entity with this component will be ignored by [`MousePosPlugin`].
///
/// If you add the [`ExcludeMouseTracking`] component to an entity that also has the [`MainCamera`] component, the app will panic.
/// You should remove either [`ExcludeMouseTracking`] or [`MainCamera`], as they should not be used together.
#[derive(Debug, Clone, Copy, Component)]
pub struct ExcludeMouseTracking;

/// The location of the mouse in screenspace.  
/// This will be updated every frame during [`CoreStage::First`]. Any systems that rely
/// on this should come after `CoreStage::First`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
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

type NeedsScreenspaceTracking = (Without<MousePos>, Without<ExcludeMouseTracking>);
type NeedsWorldspaceTracking = (Without<MousePosWorld>, Without<ExcludeMouseTracking>);

fn add_pos_components(
    cameras1: Query<(Entity, &Camera), NeedsScreenspaceTracking>,
    cameras2: Query<Entity, (With<Camera>, NeedsWorldspaceTracking)>,
    windows: Res<Windows>,
    mut commands: Commands,
) {
    for (e, camera) in cameras1.iter() {
        if let RenderTarget::Window(window_id) = camera.target {
            // get the initial position of the cursor.
            let position = windows
                .get(window_id)
                .and_then(|w| w.cursor_position())
                .unwrap_or_default();
            commands.entity(e).insert(MousePos(position));
        }
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
        for (_, mut pos) in cameras
            .iter_mut()
            .filter(|(c, ..)| c.target == RenderTarget::Window(id))
        {
            pos.0 = position;
        }
    }
}

/// The location of the mouse in worldspace.  
/// This will be updated every frame during [`CoreStage::First`]. Any systems that rely
/// on this should come after `CoreStage::First`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Component)]
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
    cameras: Query<(&GlobalTransform, &OrthographicProjection), Without<ExcludeMouseTracking>>,
) {
    for (camera, mut world, screen) in tracking.iter_mut() {
        let (camera, proj) = cameras.get(camera).expect("only orthographic cameras are supported -- consider adding an ExcludeMouseTracking component");
        let offset = Vec2::new(proj.left, proj.bottom);

        // Must multiply by projection scale before applying camera global transform
        // Otherwise you get weird offset mouse positions when both scaling and panning the camera.
        world.0 = camera.mul_vec3(((screen.0 + offset) * proj.scale).extend(0.0));
    }
}

/// Resource that specifies the main camera. If this resource is not defined, all cameras will be treated equally.
pub struct MainCamera(pub Entity);

fn update_resources(
    mut screen_res: ResMut<MousePos>,
    mut world_res: ResMut<MousePosWorld>,
    main: Option<Res<MainCamera>>,
    screen: Query<&MousePos, Changed<MousePos>>,
    world: Query<&MousePosWorld, Changed<MousePosWorld>>,
    mut main_defined_last_frame: Local<bool>,
) {
    let main = if let Some(main) = main {
        *main_defined_last_frame = true;
        main.0
    } else {
        // If the main camera was unset since last frame, zero out the resources.
        if *main_defined_last_frame {
            *screen_res = MousePos::default();
            *world_res = MousePosWorld::default();
            *main_defined_last_frame = false;
        }
        return;
    };

    if let Ok(&screen) = screen.get(main) {
        *screen_res = screen;
    }
    if let Ok(&world) = world.get(main) {
        *world_res = world;
    }
}
