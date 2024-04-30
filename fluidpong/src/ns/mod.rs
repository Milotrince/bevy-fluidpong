pub mod fluid;
pub mod math;
mod pongfluid;

use crate::{
    ns::fluid::*,
    ns::math::{fluid_step, index},
    simui::text_input,
    simui::{FluidSimVars, SimVariable},
};
use bevy::{
    input::keyboard::KeyboardInput,
    input::mouse::MouseMotion,
    input::ButtonState,
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
            .add_systems(Startup, init_fluid)
            .add_systems(Update, update_fluid);
        if self.debug {
            app.add_systems(Update, (update_simvars, update_interactive));
        }
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FluidGridMaterial {
    #[uniform(0)]
    width: f32,
    #[uniform(1)]
    height: f32,
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
    let fluid: Fluid = Fluid::new(FLUID_SIZE);
    let simvars = FluidSimVars {
        initialized: false,
        interact_mode: false,
        paused: false,
        debug: false,
        map: HashMap::from([
            ("dt".to_string(), 0.00001),
            ("iter".to_string(), 4.0),
            ("viscosity".to_string(), 0.2),
            ("diffusion".to_string(), 10.0),
            ("interact_force".to_string(), 1000.0),
            ("interact_velocity".to_string(), 0.0),
            ("diffusion".to_string(), 0.001),
        ]),
    };
    let cells = fluid.get_cells();

    commands.spawn((
        fluid,
        simvars,
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(WIDTH, HEIGHT))),
            material: materials.add(FluidGridMaterial {
                width: WIDTH,
                height: HEIGHT,
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
            let point = Vec3::new(world_position.x, world_position.y, 0.0);

            if mb.pressed(MouseButton::Left) {
                gizmos.circle_2d(world_position, 10., Color::BLUE);
                let strength = simvars.get("interact_force");
                let ix = ((point.x + WIDTH / 2.0) / (WIDTH as f32) * (FLUID_SIZE as f32)) as u32;
                let iy = ((point.y + HEIGHT / 2.0) / (HEIGHT as f32) * (FLUID_SIZE as f32)) as u32;

                fluid.add_density(ix, iy, strength);
                for motion in motion_er.read() {
                    fluid.add_velocity(
                        ix,
                        iy,
                        motion.delta.x * INTERACT_VELOCITY,
                        -motion.delta.y * INTERACT_VELOCITY,
                    );
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
        for i in 0..FLUID_SIZE {
            for j in 0..FLUID_SIZE {
                if fluid.density[index(FLUID_SIZE, i, j)] > 0.0 {
                    fluid.add_density(i, j, -dissipation);
                }
            }
        }

        if let Some(material) = materials.get_mut(&*handle) {
            material.cells = fluid.get_cells();
        }
    }
}

fn update_simvars(
    mut key_evr: EventReader<KeyboardInput>,
    mut fluidquery: Query<(&mut FluidSimVars, &mut Fluid)>,
    query: Query<(&SimVariable, &text_input::TextInputValue)>,
) {
    let (mut simvars, mut fluid) = fluidquery.single_mut();
    let mut do_update = false;
    for ev in key_evr.read() {
        if ev.state == ButtonState::Released {
            if ev.key_code == KeyCode::Enter {
                do_update = true;
            }
            if ev.key_code == KeyCode::KeyD {
                simvars.debug = !simvars.debug;
                println!("debug: {}", simvars.debug);
            }
            if ev.key_code == KeyCode::KeyP {
                simvars.paused = !simvars.paused;
                println!("paused: {}", simvars.paused);
            }
            if ev.key_code == KeyCode::KeyI {
                simvars.interact_mode = !simvars.interact_mode;
                println!("interact mode: {}", simvars.interact_mode);
            }
            if ev.key_code == KeyCode::KeyR {
                fluid.reset();
                println!("resetting")
            }
        }
    }
    if !simvars.initialized {
        do_update = true;
        simvars.initialized = true;
    }
    if do_update {
        for (simvar, input) in query.iter() {
            let value = input.0.parse::<f32>().unwrap_or(0.0);
            simvars.set(simvar.name.clone(), value);
            println!("updating {} to {}", simvar.name.clone(), value);
        }
    }
}
