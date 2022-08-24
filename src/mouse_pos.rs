use std::{fmt::Display, ops::Deref};

use bevy::{prelude::*, render::camera::RenderTarget};

/// Plugin that tracks the mouse location.
pub struct MousePosPlugin;

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, add_dependent_pos);
        app.add_system_to_stage(CoreStage::Last, add_dependent_pos);

        app.add_system_to_stage(CoreStage::First, update_pos);
        app.add_system_to_stage(CoreStage::First, update_pos_ortho.after(update_pos));

        app.init_resource::<MousePos>();
        app.init_resource::<MousePosWorld>();
        app.add_system_to_stage(CoreStage::First, update_resources.after(update_pos_ortho));
    }
}

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

fn add_dependent_pos(
    needs_screen_pos: Query<Entity, (With<MousePosWorld>, Without<MousePos>)>,
    mut commands: Commands,
) {
    for id in &needs_screen_pos {
        commands.entity(id).insert(MousePos::default());
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
    cameras: Query<(&GlobalTransform, &OrthographicProjection)>,
) {
    for (camera, mut world, screen) in tracking.iter_mut() {
        let (camera, proj) = cameras
            .get(camera)
            .expect("only orthographic cameras are supported");
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
