use std::{fmt::Display, ops::Deref};

use bevy::{
    ecs::{
        system::{Command, EntityCommands},
        world::EntityMut,
    },
    prelude::*,
    render::camera::RenderTarget,
};

/// Plugin that tracks the mouse location.
pub struct MousePosPlugin;

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::First, update_pos);
        app.add_system_to_stage(CoreStage::First, update_pos_ortho.after(update_pos));

        app.insert_resource(MousePos(default()));
        app.insert_resource(MousePosWorld(default()));
        app.add_system_to_stage(CoreStage::First, update_resources.after(update_pos_ortho));
    }
}

/// Extension trait for [`EntityCommands`] and [`EntityMut`] that allows adding mouse tracking to a camera entity.
pub trait InsertExt {
    fn add_mouse_tracking(&mut self) -> &mut Self;
    fn add_world_tracking(&mut self) -> &mut Self;
}

impl<'w> InsertExt for EntityMut<'w> {
    fn add_mouse_tracking(&mut self) -> &mut Self {
        // SAFETY: We call `self.update_location()` immediately after `self.world_mut()`,
        // so even if the location changed, that's okay.
        AddMouseTracking(self.id()).write(unsafe { self.world_mut() });
        self.update_location();
        self
    }
    fn add_world_tracking(&mut self) -> &mut Self {
        // SAFETY: We call `self.update_location()` immediately after `self.world_mut()`,
        // so even if the location changed, that's okay.
        AddWorldTracking(self.id()).write(unsafe { self.world_mut() });
        self.update_location();
        self
    }
}

impl<'w, 's, 'a> InsertExt for EntityCommands<'w, 's, 'a> {
    fn add_mouse_tracking(&mut self) -> &mut Self {
        let cmd = AddMouseTracking(self.id());
        self.commands().add(cmd);
        self
    }
    fn add_world_tracking(&mut self) -> &mut Self {
        let cmd = AddWorldTracking(self.id());
        self.commands().add(cmd);
        self
    }
}

/// The location of the mouse in screenspace.  
/// This will be updated every frame during [`CoreStage::First`]. Any systems that rely
/// on this should come after `CoreStage::First`.
#[derive(Debug, Clone, Copy, PartialEq, Component)]
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

/// A [`Command`] that adds the [`MousePos`] component to a [`Camera`], ensuring that the initial cursor position is correct.
///
/// The simplest way to enable mouse tracking for a camera is to add the component `MousePos::default`
/// -- however, doing this means that the initial value for the cursor position will be zero. This command handles that automatically.
///
/// To add both `MousePos` and [`MousePosWorld`], consider using [`AddWorldTracking`].
pub struct AddMouseTracking(Entity);

impl AddMouseTracking {
    pub const fn new(id: Entity) -> Self {
        Self(id)
    }
}

impl Command for AddMouseTracking {
    fn write(self, world: &mut World) {
        #[track_caller]
        #[cold]
        fn no_camera(camera: impl std::fmt::Debug) -> ! {
            panic!("tried to call the command `AddMouseTracking` on non-camera entity '{camera:?}'")
        }
        #[track_caller]
        #[cold]
        fn image_camera(camera: impl std::fmt::Debug) -> ! {
            panic!(
                "tried to call the command `AddMouseTracking` on a camera ({camera:?}) that renders to an image",
            )
        }
        #[track_caller]
        #[cold]
        fn no_window(window: impl std::fmt::Debug) -> ! {
            panic!("could not find the window '{window:?}'")
        }

        let camera_id = self.0;
        let camera = world
            .entity(camera_id)
            .get::<Camera>()
            .unwrap_or_else(|| no_camera(camera_id));
        let window = match camera.target {
            RenderTarget::Window(id) => id,
            RenderTarget::Image(_) => image_camera(camera_id),
        };
        let window = world
            .resource::<Windows>()
            .get(window)
            .unwrap_or_else(|| no_window(window));
        let mouse_pos = window.cursor_position().unwrap_or_default();
        world.entity_mut(camera_id).insert(MousePos(mouse_pos));
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
#[derive(Debug, Clone, Copy, PartialEq, Component)]
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

/// A [`Command`] that adds the component [`MousePosWorld`] to an entity, with a correct initial position.
/// For more details, see the docs for [`AddMouseTracking`].
pub struct AddWorldTracking(Entity);

impl AddWorldTracking {
    pub fn new(id: Entity) -> Self {
        Self(id)
    }
}

impl Command for AddWorldTracking {
    fn write(self, world: &mut World) {
        fn no_transform(id: impl std::fmt::Debug) -> ! {
            panic!("tried to call the command `AddWorldTracking` on a camera ({id:?}) with no `GlobalTransform`")
        }
        fn no_proj(id: impl std::fmt::Debug) -> ! {
            panic!("tried to call the command `AddWorldTracking` on a camera ({id:?}) with no `OrthographicProjection`")
        }

        let camera_id = self.0;

        AddMouseTracking(camera_id).write(world);

        let camera = world.entity(camera_id);
        let screen_pos = camera.get::<MousePos>().unwrap();
        let transform = camera
            .get::<GlobalTransform>()
            .unwrap_or_else(|| no_transform(camera_id));
        let proj = camera
            .get::<OrthographicProjection>()
            .unwrap_or_else(|| no_proj(camera_id));
        let world_pos = compute_world_pos_ortho(screen_pos.0, transform, proj);
        world.entity_mut(camera_id).insert(MousePosWorld(world_pos));
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
        world.0 = compute_world_pos_ortho(screen.0, camera, proj);
    }
}

fn compute_world_pos_ortho(
    screen_pos: Vec2,
    transform: &GlobalTransform,
    proj: &OrthographicProjection,
) -> Vec3 {
    let offset = Vec2::new(proj.left, proj.bottom);
    // Must multiply by projection scale before applying camera global transform
    // Otherwise you get weird offset mouse positions when both scaling and panning the camera.
    transform.mul_vec3(((screen_pos + offset) * proj.scale).extend(0.0))
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
            *screen_res = MousePos(default());
            *world_res = MousePosWorld(default());
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
