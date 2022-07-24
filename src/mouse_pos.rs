use std::{fmt::Display, ops::Deref};

use bevy::prelude::*;

/// Plugin that tracks the mouse location.
pub enum MousePosPlugin {
    /// Configuration for apps that have a single main camera.
    /// Provides global Resources for [`MousePos`] and [`MousePosWorld`].
    SingleCamera,
    /// Configuration for apps that have multiple cameras which must be handled separately.
    MultiCamera,
}

impl Plugin for MousePosPlugin {
    fn build(&self, app: &mut App) {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
        enum MouseSystem {
            ScreenPos,
            WorldPos,
            FindMain,
        }

        // System to add mouse tracking components.
        // Runs once at the end of each frame. This means that no cameras will have
        // mouse tracking components until after the first frame.
        // This might cause some issues, but it's probably for the best since,
        // during the first frame, nothing has been rendered yet.
        app.add_system_to_stage(CoreStage::PostUpdate, add_pos_components);

        app.add_system_to_stage(CoreStage::First, update_pos.label(MouseSystem::ScreenPos));
        app.add_system_to_stage(
            CoreStage::First,
            update_pos_ortho
                .label(MouseSystem::WorldPos)
                .after(MouseSystem::ScreenPos),
        );

        match self {
            Self::SingleCamera => {
                app.insert_resource(MousePos(Default::default()));
                app.insert_resource(MousePosWorld(Default::default()));
                app.init_resource::<MainCameraStore>();

                // system to update the current main camera
                app.add_system_set_to_stage(
                    CoreStage::First,
                    SystemSet::new()
                        .label(MouseSystem::FindMain)
                        .with_run_criteria(main_camera_changed)
                        .with_system(find_main_camera),
                );
                //
                app.add_system_to_stage(
                    CoreStage::First,
                    update_resources
                        .after(MouseSystem::WorldPos)
                        .after(MouseSystem::FindMain),
                );
            }
            Self::MultiCamera => {}
        }
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

fn add_pos_components(
    cameras1: Query<(Entity, &Camera), Without<MousePos>>,
    cameras2: Query<Entity, (With<Camera>, Without<MousePosWorld>)>,
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

fn update_pos_ortho(
    mut tracking: Query<(Entity, &mut MousePosWorld, &MousePos), Changed<MousePos>>,
    cameras: Query<(&GlobalTransform, &OrthographicProjection)>,
) {
    for (camera, mut world, screen) in tracking.iter_mut() {
        let (camera, proj) = cameras
            .get(camera)
            .expect("only orthographic cameras are supported");
        let offset = Vec2::new(proj.left, proj.bottom);
        world.0 = camera.mul_vec3((screen.0 + offset).extend(0.0)) * proj.scale;
    }
}

/// Marker component for the primary camera in the world.
/// If only one camera exists, this is optional.
#[derive(Debug, Clone, Copy, Component)]
pub struct MainCamera;

/// Resource that caches the main camera, so it doesn't need to be looked up every frame.
/// This is an implementation detail and thus should not be part of the public api.
#[derive(Debug, Default)]
struct MainCameraStore(Option<Entity>);

// only run when the candidates for the main camera change.
use bevy::ecs::schedule::ShouldRun;
use bevy::render::camera::RenderTarget;

fn main_camera_changed(
    cam: Query<Entity, Added<Camera>>,
    main: Query<Entity, Added<MainCamera>>,
) -> ShouldRun {
    if !cam.is_empty() || !main.is_empty() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn find_main_camera(
    mut main_store: ResMut<MainCameraStore>,
    cameras: Query<(Entity, Option<&MainCamera>), With<Camera>>,
) {
    use bevy::ecs::query::QuerySingleError;
    main_store.0 = match cameras.get_single() {
        Ok((e, ..)) => Some(e),
        Err(QuerySingleError::NoEntities(_)) => {
            // no main camera exists
            None
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            // try to disambiguate
            let mut mains = cameras.iter().filter_map(|(e, main)| main.and(Some(e)));
            let main = mains.next().unwrap_or_else(|| {
                panic!("cannot identify main camera -- consider adding the MainCamera component to one of the cameras")
            });
            if mains.next().is_some() {
                panic!("only one camera may be marked with the MainCamera component");
            }
            Some(main)
        }
    }
}

fn update_resources(
    mut screen_res: ResMut<MousePos>,
    mut world_res: ResMut<MousePosWorld>,
    main: Res<MainCameraStore>,
    screen: Query<&MousePos, Changed<MousePos>>,
    world: Query<&MousePosWorld, Changed<MousePosWorld>>,
) {
    let main = match main.0 {
        Some(m) => m,
        None => return, // no main camera, try again next frame
    };
    // update the global resources if the components for the main camera changed.
    if let Ok(screen) = screen.get(main) {
        screen_res.0 = screen.0;
    }
    if let Ok(world) = world.get(main) {
        world_res.0 = world.0;
    }
}
