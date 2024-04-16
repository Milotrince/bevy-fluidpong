use bevy::{
    prelude::*,
    sprite::Material2dPlugin
};
pub mod sphfluid;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<sphfluid::MetaballMaterial>::default(),
            sphfluid::SPHFluidPlugin
        ))
        .run();
}