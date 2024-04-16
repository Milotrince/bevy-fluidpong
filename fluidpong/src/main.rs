use bevy::{
    prelude::*,
    sprite::Material2dPlugin
};
pub mod fluid;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<fluid::FluidMaterial>::default(),
            fluid::FluidPlugin
        ))
        .run();
}