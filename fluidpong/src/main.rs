use bevy::prelude::*;
pub mod pong;
pub mod sphfluid;

fn main() {
    App::new()
        .add_systems(Startup, spawn_camera)
        .add_plugins((
            DefaultPlugins,
            sphfluid::SPHFluidPlugin,
            // pong::PongPlugin
        ))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2dBundle::default());
}
