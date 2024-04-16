use bevy::prelude::*;
pub mod pong;
pub mod sphfluid;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, sphfluid::SPHFluidPlugin, pong::PongPlugin))
        .run();
}
