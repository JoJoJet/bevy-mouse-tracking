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
                //
                app.add_system_to_stage(
                    CoreStage::First,
                    update_main_camera.after(MouseSystem::WorldPos),
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
        // get the initial position of the cursor.
        let position = windows
            .get(camera.window)
            .and_then(|w| w.cursor_position())
            .unwrap_or_default();
        commands.entity(e).insert(MousePos(position));
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
        for (_, mut pos) in cameras.iter_mut().filter(|(c, ..)| c.window == id) {
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
        world.0 =
            camera.mul_vec3(screen.0.extend(0.0) + Vec3::new(proj.left, proj.bottom, proj.near));
    }
}

/// Marker component for the primary camera in the world.
/// If only one camera exists, this is optional.
#[derive(Debug, Clone, Copy, Component)]
pub struct MainCamera;

fn update_main_camera(
    mut screen_res: ResMut<MousePos>,
    mut world_res: ResMut<MousePosWorld>,
    cameras: Query<(&MousePos, &MousePosWorld, Option<&MainCamera>)>,
) {
    use bevy::ecs::system::QuerySingleError;
    let (screen, world, ..) = match cameras.get_single() {
        Ok(x) => x,
        Err(QuerySingleError::NoEntities(_)) => {
            // this is okay, try again next frame.
            return;
        }
        Err(QuerySingleError::MultipleEntities(_)) => {
            // try to disambiguate
            let mut mains = cameras.iter().filter(|(.., main)| main.is_some());

            let main = mains.next().unwrap_or_else(||panic!("cannot identify main camera -- consider adding the MainCamera component to one of the cameras") );
            if mains.next().is_some() {
                panic!("only one camera may be marked with the MainCamera component");
            }
            main

            // ambiguous! very bad
        }
    };
    screen_res.0 = screen.0;
    world_res.0 = world.0;
}
