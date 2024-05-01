pub mod fluid;
pub mod math;
mod pongfluid;

use crate::{
    ns::fluid::*,
    ns::math::{fluid_step, index},
    simui::FluidSimVars,
};
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::PrimaryWindow,
};

pub struct FluidPlugin {
    pub debug: bool,
}

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<FluidGridMaterial>::default())
            .add_systems(PostStartup, init_fluid)
            .add_systems(Update, update_fluid);
        if self.debug {
            app.add_systems(Update, update_interactive);
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FluidGridMaterial {
    #[uniform(0)]
    screen_size: Vec2,
    #[uniform(1)]
    grid_size: Vec2,
    #[uniform(2)]
    cells: [Vec4; NUM_CELLS],
}

impl Material2d for FluidGridMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/fluidgrid.wgsl".into()
    }
}

fn init_fluid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FluidGridMaterial>>,
) {
    let fluid: Fluid = Fluid::new();
    let simvars = FluidSimVars::new(HashMap::from([
        ("dt".to_string(), 0.00001),
        ("iter".to_string(), 4.0),
        ("viscosity".to_string(), 0.2),
        ("diffusion".to_string(), 10.0),
        ("interact_force".to_string(), 1000.0),
        ("interact_velocity".to_string(), 0.0),
        ("diffusion".to_string(), 0.001),
    ]));
    let cells = fluid.get_cells();

    commands.spawn((
        fluid,
        simvars,
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(WIDTH, HEIGHT))),
            material: materials.add(FluidGridMaterial {
                screen_size: Vec2::new(WIDTH as f32, HEIGHT as f32),
                grid_size: Vec2::new(GRID_X as f32, GRID_Y as f32),
                cells: cells,
            }),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
    ));
}

fn update_interactive(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mb: Res<ButtonInput<MouseButton>>,
    mut motion_er: EventReader<MouseMotion>,
    mut query: Query<(&mut Fluid, &FluidSimVars)>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let (mut fluid, simvars) = query.single_mut();

    let window: &Window = window_query.single();
    if let Some(cursor_position) = window.cursor_position() {
        if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        {
            let point = Vec2::new(world_position.x, world_position.y);

            if mb.pressed(MouseButton::Left) {
                gizmos.circle_2d(world_position, 10., Color::WHITE);
                let strength = simvars.get("interact_force");
                fluid.add_density(point, strength);
                for motion in motion_er.read() {
                    fluid
                        .add_velocity(point, Vec2::new(1., -1.) * motion.delta * INTERACT_VELOCITY);
                }
            }
        }
    }
}

fn update_fluid(
    mut query: Query<(&mut Fluid, &Handle<FluidGridMaterial>, &FluidSimVars)>,
    mut materials: ResMut<Assets<FluidGridMaterial>>,
) {
    let (mut fluid, handle, simvars) = query.single_mut();
    let dissipation = simvars.get("dissipation");
    let viscosity = simvars.get("viscosity");
    let diffusion = simvars.get("diffusion");
    let dt = simvars.get("dt");
    let iter = simvars.get("iter") as u32;
    if !simvars.paused {
        fluid_step(&mut fluid, viscosity, diffusion, dt, iter);
        for i in 0..GRID_X {
            for j in 0..GRID_Y {
                if fluid.density[index(i, j)] > dissipation {
                    fluid.add_density_grid(i, j, -dissipation);
                }
            }
        }

        if let Some(material) = materials.get_mut(&*handle) {
            material.cells = fluid.get_cells();
        }
    }
}