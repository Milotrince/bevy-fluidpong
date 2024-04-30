use bevy::prelude::*;
pub mod sph;
pub mod pong;
pub mod ns;
pub mod simui;

const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;
const GAME_WIDTH: f32 = SCREEN_WIDTH;
const GAME_HEIGHT: f32 = SCREEN_HEIGHT - 160.0;

fn main() {
    App::new()
        .add_systems(Startup, (spawn_camera, resize_window))
        .add_plugins((
            DefaultPlugins,
            // COMMENT/UNCOMMENT FOR WHAT YOU ARE WORKING ON
            pong::PongPlugin,
            sph::SPHFluidPlugin,
            // ns::FluidPlugin,
            // simui::SimUIPlugin,
        ))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2dBundle::default());
}

fn resize_window(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.resolution.set(SCREEN_WIDTH, SCREEN_HEIGHT);
}
