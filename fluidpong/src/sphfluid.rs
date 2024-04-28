use crate::lib::{
    kernel::{Kernel, KernelFunction, Poly6Kernel, SpikyKernel, ViscosityKernel},
    spatial_grid::{Position, SpatialGrid2D},
    text_input,
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

const INITIAL_DENSITY: f32 = 100.0;
#[derive(Component, Clone)]
pub struct FluidSimVars {
    map: HashMap<String, f32>,
    initialized: bool,
    paused: bool,
}

const PARTICLE_SIZE: f32 = 2.0;
const NUM_PARTICLES_X: u32 = 32; //64;
const NUM_PARTICLES_Y: u32 = 32; //64;
const NUM_PARTICLES: usize = (NUM_PARTICLES_X * NUM_PARTICLES_Y) as usize;

const PARTICLES_DX: f32 = 4.0;
const PARTICLES_DY: f32 = 4.0;
const PARTICLE_MASS: f32 = 100.0;

// use smaller when testing
static WALL_X: f32 = 200.0;
static WALL_Y: f32 = 200.0;

pub struct SPHFluidPlugin;

impl Plugin for SPHFluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<MetaballMaterial>::default())
            .add_systems(Startup, init_fluid)
            .add_systems(
                Update,
                (update_simvars, update_interactive, update_fluid).chain(),
            );
    }
}

#[derive(Debug, Clone)]
struct Particle {
    position: Vec2,
    velocity: Vec2,
    mass: f32,
    density: f32,
    pressure: f32,
    ext_force: Vec2,
}

impl Position for Particle {
    fn position(&self) -> Vec2 {
        self.position
    }
}

#[derive(Component)]
struct SPHFluid {
    particles: SpatialGrid2D<Particle>,
    density_kernel: Kernel,
    pressure_kernel: Kernel,
    viscosity_kernel: Kernel,
}

impl SPHFluid {
    fn new() -> Self {
        let mut fluid = SPHFluid {
            // These numbers must all be the same. Sorry Trinity I got rid of
            // the simvar for this param :( but we can add it back if you want
            particles: SpatialGrid2D::new(10.0),
            density_kernel: Poly6Kernel::new(10.0).into(),
            pressure_kernel: SpikyKernel::new(10.0).into(),
            viscosity_kernel: ViscosityKernel::new(10.0).into(),
        };

        let hx: f32 = PARTICLES_DX * NUM_PARTICLES_X as f32 / 2.0;
        let hy: f32 = PARTICLES_DY * NUM_PARTICLES_Y as f32 / 2.0;

        for i in 1..NUM_PARTICLES_X {
            for j in 1..NUM_PARTICLES_Y {
                let particle = Particle {
                    position: Vec2::new(i as f32 * PARTICLES_DX - hx, j as f32 * PARTICLES_DY - hy),
                    velocity: Vec2::ZERO,
                    mass: PARTICLE_MASS,
                    density: INITIAL_DENSITY,
                    pressure: 0.0,
                    ext_force: Vec2::ZERO,
                };
                fluid.particles.insert(particle);
            }
        }

        fluid
    }

    fn update_particle_forces(&mut self, dt: f32, simvars: &FluidSimVars) {
        if let (
            Some(&viscosity),
            Some(&pressure_coeff),
            Some(&wall_x),
            Some(&wall_y),
            Some(&restitution),
            Some(&friction),
            Some(&gravity),
            Some(&sim_speed),
        ) = (
            simvars.map.get("viscosity"),
            simvars.map.get("pressure"),
            simvars.map.get("wall_x"),
            simvars.map.get("wall_y"),
            simvars.map.get("restitution"),
            simvars.map.get("friction"),
            simvars.map.get("gravity"),
            simvars.map.get("sim_speed"),
        ) {
            let dt = dt * sim_speed;
            let old_particles = self.particles.clone();

            // Calculate density and pressure. Density is calculated based on
            // the mass of the particles within the smoothing radius of the
            // particle. Pressure is then calculated based on the density.
            for particle in self.particles.iter_mut() {
                particle.density = 0.0;
                for neighbor in old_particles.query(particle.position) {
                    if particle.position == neighbor.position {
                        continue;
                    }
                    let r = neighbor.position - particle.position;
                    particle.density += neighbor.mass * self.density_kernel.evaluate(r);
                }
                particle.pressure = pressure_coeff * (particle.density - 0.5).max(0.0);
            }

            // FIXME: Don't clone all the time
            let old_particles = self.particles.clone();

            // Calculate forces for each particle based on the density and
            // pressure of the particles within the smoothing radius.
            for particle in self.particles.iter_mut() {
                let mut pressure_force = Vec2::ZERO;
                let mut viscosity_force = Vec2::ZERO;

                for neighbor in old_particles.query(particle.position) {
                    if particle.position == neighbor.position {
                        continue;
                    }
                    let r = particle.position - neighbor.position;

                    pressure_force -= neighbor.mass / neighbor.density
                        * (particle.pressure + neighbor.pressure)
                        / 2.0
                        * self.pressure_kernel.gradient(r);

                    viscosity_force += viscosity * neighbor.mass / neighbor.density
                        * (neighbor.velocity - particle.velocity)
                        * self.viscosity_kernel.laplacian(r);
                }

                let total_force = pressure_force
                    + viscosity_force
                    + particle.ext_force
                    + Vec2::new(0.0, -gravity * particle.mass);
                let acceleration = total_force / particle.mass;
                let velocity = particle.velocity + acceleration * dt;

                particle.velocity = velocity;
                particle.position += velocity * dt;

                // Collision with walls
                // particle.position.x = particle.position.x.max(-wall_x).min(wall_x);
                // particle.position.y = particle.position.y.max(-wall_y).min(wall_y);

                let mut collision_normal = Vec2::ZERO;
                if particle.position.x < -wall_x {
                    collision_normal += Vec2::X;
                    particle.position.x = -wall_x;
                }
                if particle.position.x > wall_x {
                    collision_normal -= Vec2::X;
                    particle.position.x = wall_x;
                }
                if particle.position.y < -wall_y {
                    collision_normal += Vec2::Y;
                    particle.position.y = -wall_y;
                }
                if particle.position.y > wall_y {
                    collision_normal -= Vec2::Y;
                    particle.position.y = wall_y;
                }

                if collision_normal != Vec2::ZERO {
                    collision_normal = collision_normal.normalize();
                    let velocity_normal =
                        particle.velocity.dot(collision_normal) * collision_normal;
                    let velocity_tangential = particle.velocity - velocity_normal;

                    particle.velocity =
                        -restitution * velocity_normal + (1.0 - friction) * velocity_tangential;
                }
            }

            self.particles.recompute();
        }
    }

    fn set_external_force(&mut self, point: Vec2, force: Vec2) {
        for particle in self.particles.iter_mut() {
            let distance: f32 = particle.position.distance(point);
            let logistic_response = 1.0 / (1.0 + f32::exp(1.0 + distance));
            particle.ext_force = force * logistic_response;
        }
    }

    fn get_balls(&self) -> [Vec4; NUM_PARTICLES] {
        let mut balls = [Vec4::ZERO; NUM_PARTICLES];
        for (i, particle) in self.particles.iter().enumerate() {
            if i >= NUM_PARTICLES {
                break;
            }
            balls[i] = Vec4::new(
                particle.position.x,
                particle.position.y,
                particle.density,
                particle.velocity.length(),
            );
        }
        balls
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct MetaballMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    radius: f32,
    #[uniform(2)]
    balls: [Vec4; NUM_PARTICLES],
}

impl Material2d for MetaballMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/metaball.wgsl".into()
    }
}

fn init_fluid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MetaballMaterial>>,
) {
    let fluid: SPHFluid = SPHFluid::new();
    let balls = fluid.get_balls();
    let simvars = FluidSimVars {
        initialized: false,
        paused: false,
        map: HashMap::from([
            ("gravity".to_string(), 0.0),
            ("restitution".to_string(), 0.0),
            ("friction".to_string(), 0.0),
            ("viscosity".to_string(), 0.0),
            ("interact_force".to_string(), 0.0),
            ("interact_radius".to_string(), 0.0),
            ("dist_threshold".to_string(), 0.0),
            ("wall_x".to_string(), 0.0),
            ("wall_y".to_string(), 0.0),
        ]),
    };

    commands.spawn((
        fluid,
        simvars,
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(WALL_X * 2.0, WALL_Y * 2.0))),
            material: materials.add(MetaballMaterial {
                color: Color::BLUE,
                radius: PARTICLE_SIZE,
                balls: balls,
            }),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
    ));
}

fn update_fluid(
    time: Res<Time>,
    mut query: Query<(&mut SPHFluid, &mut Handle<MetaballMaterial>, &FluidSimVars)>,
    mut materials: ResMut<Assets<MetaballMaterial>>,
    mut gizmos: Gizmos,
) {
    let (mut fluid, handle, simvars) = query.single_mut();
    if !simvars.paused {
        let dt: f32 = time.delta_seconds();
        fluid.update_particle_forces(dt, simvars);

        if let Some(material) = materials.get_mut(&*handle) {
            material.balls = fluid.get_balls();
        }
    }

    // DEBUG
    for particle in fluid.particles.iter() {
        gizmos.circle_2d(
            Vec2::new(particle.position.x, particle.position.y),
            1.,
            Color::rgba(1.0, 1.0, 1.0, 1.00),
        );
    }
}

fn update_interactive(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut motion_er: EventReader<MouseMotion>,
    mut query: Query<(&mut SPHFluid, &FluidSimVars)>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();

    let (mut fluid, simvars) = query.single_mut();
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
                if let Some(force_coeff) = simvars.map.get("interact_force") {
                    fluid.set_external_force(point, force * *force_coeff);
                }
            }
        }
    }
}

fn update_simvars(
    mut key_evr: EventReader<KeyboardInput>,
    mut fluidquery: Query<(&mut FluidSimVars, &mut SPHFluid)>,
    query: Query<(&crate::simui::SimVariable, &text_input::TextInputValue)>,
) {
    let (mut simvars, mut fluid) = fluidquery.single_mut();
    let mut do_update = false;
    for ev in key_evr.read() {
        if ev.state == ButtonState::Released {
            if ev.key_code == KeyCode::Enter {
                do_update = true;
            }
            if ev.key_code == KeyCode::KeyP {
                simvars.paused = !simvars.paused;
            }
            if ev.key_code == KeyCode::KeyR {
                *fluid = SPHFluid::new();
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
            simvars.map.insert(simvar.name.clone(), value);
            println!("updating {} to {}", simvar.name.clone(), value);
        }
    }
}
