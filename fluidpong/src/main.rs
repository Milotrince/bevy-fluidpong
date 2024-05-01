use bevy::prelude::*;
use clap::Parser;

pub mod sph;
pub mod pong;
pub mod ns;
pub mod simui;

const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;
const GAME_WIDTH: f32 = SCREEN_WIDTH;
const GAME_HEIGHT: f32 = SCREEN_HEIGHT - 160.0;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    fluid: String,

    #[arg(short, long, default_value_t = false)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    let mut app = App::new();
    app.add_systems(Startup, (spawn_camera, resize_window));
    app.add_plugins(DefaultPlugins);
    app.insert_resource(ClearColor(Color::BLACK));
    if args.fluid == "sph" {
        app.add_plugins(sph::FluidPlugin {debug: args.debug});
    } else {
        app.add_plugins(ns::FluidPlugin {debug: args.debug});
    }
    app.add_plugins(pong::PongPlugin);
    if args.debug {
        app.add_plugins(simui::SimUIPlugin {fluid_type: args.fluid});
    }
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty().insert(Camera2dBundle::default());
}

fn resize_window(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.resolution.set(SCREEN_WIDTH, SCREEN_HEIGHT);
}
