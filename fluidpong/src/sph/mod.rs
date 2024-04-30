pub mod fluid;
pub mod kernel;
pub mod particle;
mod pongfluid;
pub mod spatial_grid;

use bevy::app::{App, Plugin, Startup, Update};
use bevy::ecs::event::EventReader;
use bevy::ecs::query::With;
use bevy::ecs::system::{Commands, Query, Res};
use bevy::gizmos::gizmos::Gizmos;
use bevy::input::mouse::MouseMotion;
use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::render::color::Color;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle};
use bevy::time::Time;
use bevy::transform::components::GlobalTransform;
use bevy::window::{PrimaryWindow, Window};

pub struct FluidPlugin {
    pub debug: bool,
}

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<MetaballMaterial>::default())
            .add_systems(PostStartup, startup)
            .add_systems(Update, (update_fluid, update_shader));
        if self.debug {
            app.add_systems(Update, (draw_gizmos, update_interactive));
        }
    }
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MetaballMaterial>>,
) {
    let fluid = fluid::Fluid::new();
    let balls = fluid.get_balls();
    commands.spawn((
        fluid,
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(
                meshes.add(Rectangle::new(fluid::WALL_X * 2.0, fluid::WALL_Y * 2.0)),
            ),
            material: materials.add(MetaballMaterial { color: Color::BLUE, balls: balls }),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
    ));
}

fn update_fluid(time: Res<Time>, mut fluid_query: Query<&mut fluid::Fluid>) {
    let mut fluid = fluid_query.single_mut();
    fluid.compute_density_pressure();
    fluid.compute_forces();
    fluid.integrate(time.delta_seconds());
}

fn update_shader(
    mut query: Query<(&fluid::Fluid, &Handle<MetaballMaterial>)>,
    mut materials: ResMut<Assets<MetaballMaterial>>,
) {
    let (fluid, handle) = query.single_mut();
    if let Some(material) = materials.get_mut(&*handle) {
        material.balls = fluid.get_balls();
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
    fluid.set_external_force(Vec2::ZERO, Vec2::ZERO, 0.0);

    for motion in motion_er.read() {
        let window: &Window = window_query.single();

        if let Some(cursor_position) = window.cursor_position() {
            if let Some(world_position) =
                camera.viewport_to_world_2d(camera_transform, cursor_position)
            {
                gizmos.circle_2d(world_position, 10., Color::WHITE);

                let point = Vec2::new(world_position.x, world_position.y);
                let force = Vec2::new(motion.delta.x, -motion.delta.y);
                fluid.set_external_force(point, force * 30000.0, 10.0);
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

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MetaballMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    balls: [Vec4; fluid::NUM_PARTICLES],
}

impl Material2d for MetaballMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/metaball.wgsl".into()
    }
}
