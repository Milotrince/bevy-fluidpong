use bevy::prelude::*;
pub mod fluid;

fn main() {
    App::new()
        .insert_resource(fluid::SpatialGrid::new())
        .add_plugins((
            DefaultPlugins,
            fluid::FluidPlugin
        ))
        .run();
}