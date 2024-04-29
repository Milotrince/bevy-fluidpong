use bevy::prelude::*;
pub mod pong;
pub mod sphfluid;
pub mod nsfluid;
pub mod nsmath;
pub mod simui;
pub mod lib;

fn main() {
    App::new()
        .add_systems(Startup, (spawn_camera, resize_window))
        .add_plugins((
            DefaultPlugins,
            // COMMENT/UNCOMMENT FOR WHAT YOU ARE WORKING ON
            // sphfluid::SPHFluidPlugin,
            nsfluid::NSFluidPlugin,
            simui::SimUIPlugin,
            // pong::PongPlugin
            // fluidpong::FluidPongPlugin,
        ))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2dBundle::default());
}
fn resize_window(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.resolution.set(640.0, 480.0);
}
