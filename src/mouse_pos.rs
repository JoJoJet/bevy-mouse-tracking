use std::{fmt::Display, ops::Deref};

use bevy::{
    ecs::system::EntityCommand,
    prelude::*,
    render::camera::RenderTarget,
    window::{PrimaryWindow, WindowRef},
};

/// Plugin that tracks the mouse location.
pub struct MousePosPlugin;

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            update_pos.after(Events::<CursorMoved>::update_system),
        );
        app.add_system_to_stage(CoreStage::First, update_pos_ortho.after(update_pos));

        app.insert_resource(MousePos(default()));
        app.insert_resource(MousePosWorld(default()));
        app.add_system_to_stage(CoreStage::First, update_resources.after(update_pos_ortho));
    }
}

/// The location of the mouse in screenspace.  
/// This will be updated every frame during [`CoreStage::First`]. Any systems that rely
/// on this should come after `CoreStage::First`.
#[derive(Debug, Resource, Clone, Copy, PartialEq, Component)]
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
/// To add both `MousePos` and [`MousePosWorld`], consider using [`InitWorldTracking`].
pub struct InitMouseTracking;

impl EntityCommand for InitMouseTracking {
    fn write(self, entity: Entity, world: &mut World) {
        #[track_caller]
        #[cold]
        fn no_camera(id: impl std::fmt::Debug) -> ! {
            panic!("tried to call the command `InitMouseTracking` on non-camera entity '{id:?}'")
        }
        #[track_caller]
        #[cold]
        fn image_camera(id: impl std::fmt::Debug) -> ! {
            panic!(
                "tried to call the command `InitMouseTracking` on a camera ({id:?}) that renders to an image",
            )
        }
        #[track_caller]
        #[cold]
        fn no_window(id: impl std::fmt::Debug) -> ! {
            panic!("could not find the window '{id:?}'")
        }

        let primary_window = world
            .query_filtered::<Entity, With<PrimaryWindow>>()
            .get_single(world)
            .ok();

        let camera = world
            .entity(entity)
            .get::<Camera>()
            .unwrap_or_else(|| no_camera(entity));
        let RenderTarget::Window(window_id) = camera.target else {
            image_camera(entity);
        };
        let window_id = window_id
            .normalize(primary_window)
            .expect("`PrimaryWindow` does not exist")
            .entity();

        let window = world
            .query::<&Window>()
            .get(world, window_id)
            .unwrap_or_else(|_| no_window(window_id));
        let mouse_pos = window.cursor_position().unwrap_or_default();

        world.entity_mut(entity).insert(MousePos(mouse_pos));
    }
}

fn update_pos(
    mut movement: EventReader<CursorMoved>,
    mut cameras: Query<(&Camera, &mut MousePos)>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_window = primary_window.get_single().ok();
    for &CursorMoved {
        window, position, ..
    } in movement.iter()
    {
        let target = RenderTarget::Window(WindowRef::Entity(window)).normalize(None);
        // find all cameras corresponding to the window on which the cursor moved.
        for (_, mut pos) in cameras
            .iter_mut()
            .filter(|(c, ..)| c.target.normalize(primary_window) == target)
        {
            pos.0 = position;
        }
    }
}

/// The location of the mouse in worldspace.  
/// This will be updated every frame during [`CoreStage::First`]. Any systems that rely
/// on this should come after `CoreStage::First`.
#[derive(Debug, Resource, Clone, Copy, PartialEq, Component)]
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
/// For more details, see the docs for [`InitMouseTracking`].
///
/// Executing this command automatically executes `InitMouseTracking`.
pub struct InitWorldTracking;

impl EntityCommand for InitWorldTracking {
    fn write(self, entity: Entity, world: &mut World) {
        fn no_transform(id: impl std::fmt::Debug) -> ! {
            panic!("tried to call the command `InitWorldTracking` on a camera ({id:?}) with no `GlobalTransform`")
        }
        fn no_proj(id: impl std::fmt::Debug) -> ! {
            panic!("tried to call the command `InitWorldTracking` on a camera ({id:?}) with no `OrthographicProjection`")
        }

        InitMouseTracking.write(entity, world);

        let mut entity_mut = world.entity_mut(entity);

        let screen_pos = entity_mut.get::<MousePos>().unwrap();
        let &transform = entity_mut
            .get::<GlobalTransform>()
            .unwrap_or_else(|| no_transform(entity));
        let proj = entity_mut
            .get::<OrthographicProjection>()
            .unwrap_or_else(|| no_proj(entity));
        let world_pos = compute_world_pos_ortho(screen_pos.0, transform, proj);
        entity_mut.insert(MousePosWorld(world_pos));
    }
}

fn update_pos_ortho(
    mut tracking: Query<
        (Entity, &mut MousePosWorld, &MousePos),
        Or<(Changed<MousePos>, Changed<GlobalTransform>)>,
    >,
    cameras: Query<(&GlobalTransform, &OrthographicProjection)>,
) {
    for (camera, mut world, screen) in tracking.iter_mut() {
        let (&camera, proj) = cameras
            .get(camera)
            .expect("only orthographic cameras are supported");
        world.0 = compute_world_pos_ortho(Vec2::new(screen.0.x, -screen.0.y), camera, proj);
    }
}

fn compute_world_pos_ortho(
    screen_pos: Vec2,
    transform: GlobalTransform,
    proj: &OrthographicProjection,
) -> Vec3 {
    let offset = Vec2::new(proj.left, proj.top);
    // Must multiply by projection scale before applying camera global transform
    // Otherwise you get weird offset mouse positions when both scaling and panning the camera.
    transform * (((screen_pos + offset) * proj.scale) * Vec2::new(1.0, -1.0)).extend(0.0)
}

/// Marker component for the main camera. If no main camera is specified, all cameras will be treated equally.
#[derive(Component)]
pub struct MainCamera;

fn update_resources(
    mut last_main: Local<Option<Entity>>,
    added_main: Query<Entity, Added<MainCamera>>,
    removed_main: RemovedComponents<MainCamera>,
    mut screen_res: ResMut<MousePos>,
    mut world_res: ResMut<MousePosWorld>,
    screen: Query<&MousePos>,
    world: Query<&MousePosWorld>,
) {
    // List of all entities known to have the MainCamera marker.
    // This includes the main camera from last frame, and all entities with the component added this frame.
    let mut with_marker: Vec<_> = Option::into_iter(*last_main).chain(&added_main).collect();
    // Ditch any removed components.
    for rem in removed_main.iter() {
        if let Some(idx) = with_marker.iter().position(|&x| x == rem) {
            with_marker.remove(idx);
        }
    }
    match *with_marker {
        // If there is only one main camera, update the resources using it.
        [main] => {
            *last_main = Some(main);
            let screen = screen.get(main).map_or_else(|_| default(), |s| s.0);
            if screen_res.0 != screen {
                screen_res.0 = screen;
            }
            let world = world.get(main).map_or_else(|_| default(), |w| w.0);
            if world_res.0 != world {
                world_res.0 = world;
            }
        }
        // If there is no main camera, zero out the resources.
        [] => {
            if last_main.is_some() {
                *last_main = None;
                *screen_res = MousePos(default());
                *world_res = MousePosWorld(default());
            }
        }
        // Panic if there is more than one main camera.
        [..] => {
            panic!("`bevy_mouse_tracking_plugin`: there cannot be more than one entity with a `MainCamera` component");
        }
    }
}
