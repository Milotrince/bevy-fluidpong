pub mod fluid;
pub mod kernel;
pub mod particle;
pub mod spatial_grid;

use bevy::app::{App, Plugin, Startup, Update};
use bevy::ecs::event::EventReader;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::gizmos::gizmos::Gizmos;
use bevy::input::mouse::MouseMotion;
use bevy::math::Vec2;
use bevy::render::camera::Camera;
use bevy::render::color::Color;
use bevy::time::Time;
use bevy::transform::components::GlobalTransform;
use bevy::window::{PrimaryWindow, Window};

pub struct SPHFluidPlugin;

impl Plugin for SPHFluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, draw_gizmos)
            .add_systems(Update, update_interactive)
            .add_systems(Update, update_fluid);
    }
}

fn startup(mut commands: Commands) {
    commands.spawn(fluid::Fluid::new());
}

fn update_fluid(time: Res<Time>, mut fluids: Query<&mut fluid::Fluid>) {
    for mut fluid in fluids.iter_mut() {
        // Update the fluid
        fluid.compute_density_pressure();
        fluid.compute_forces();
        fluid.integrate(time.delta_seconds());
    }
}

fn update_interactive(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut motion_er: EventReader<MouseMotion>,
    mut query: Query<&mut fluid::Fluid>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();

    let mut fluid = query.single_mut();
    fluid.set_external_force(Vec2::ZERO, Vec2::ZERO);

    for motion in motion_er.read() {
        let window: &Window = window_query.single();

        if let Some(cursor_position) = window.cursor_position() {
            if let Some(world_position) =
                camera.viewport_to_world_2d(camera_transform, cursor_position)
            {
                gizmos.circle_2d(world_position, 10., Color::WHITE);

                let point = Vec2::new(world_position.x, world_position.y);
                let force = Vec2::new(motion.delta.x, -motion.delta.y);
                fluid.set_external_force(point, force * 30000.0);
            }
        }
    }
}

fn draw_gizmos(mut gizmos: Gizmos, fluids: Query<&fluid::Fluid>) {
    for fluid in fluids.iter() {
        for particle in fluid.particles() {
            // Draw a circle at the particle's position
            gizmos.circle_2d(particle.position, 2.0, Color::WHITE);
        }
    }
}
