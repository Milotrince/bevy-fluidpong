use crate::lib::text_input;
use crate::simui;
use bevy::{
    input::keyboard::KeyboardInput,
    input::mouse::{MouseButtonInput, MouseMotion},
    input::ButtonState,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::PrimaryWindow,
};

const INITIAL_DENSITY: f32 = 100.0;

const PARTICLE_SIZE: f32 = 2.0;
const NUM_PARTICLES_X: u32 = 32; //64;
const NUM_PARTICLES_Y: u32 = 32; //64;
const NUM_PARTICLES: usize = (NUM_PARTICLES_X * NUM_PARTICLES_Y) as usize;

const PARTICLES_DX: f32 = 4.0;
const PARTICLES_DY: f32 = 4.0;
const PARTICLE_MASS: f32 = 10.0;
const GRID_CELL_SIZE: f32 = 10.0;

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

struct Particle {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    density: f32,
    pressure: Vec3,
    ext_force: Vec3,
}

impl Particle {
    fn to_cell(&self) -> Cell {
        return Cell {
            position: self.position,
            velocity: self.velocity,
            mass: self.mass,
        };
    }
}

#[derive(Copy, Clone)]
pub struct Cell {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
}

pub struct SpatialGrid {
    cells: HashMap<(i32, i32), Vec<Cell>>,
}

impl SpatialGrid {
    const NEIGHBOR_OFFSETS: [(i32, i32); 5] = [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)];

    pub fn new() -> Self {
        SpatialGrid {
            cells: HashMap::new(),
        }
    }

    pub fn insert(&mut self, cell: Cell) {
        let key = (
            (cell.position.x / GRID_CELL_SIZE).floor() as i32,
            (cell.position.y / GRID_CELL_SIZE).floor() as i32,
        );
        self.cells.entry(key).or_insert_with(Vec::new).push(cell);
    }

    pub fn get_neighbors(&self, position: Vec3) -> Vec<Cell> {
        let mut neighbors = Vec::new();
        let key = (
            (position.x / GRID_CELL_SIZE).floor() as i32,
            (position.y / GRID_CELL_SIZE).floor() as i32,
        );
        for offset in Self::NEIGHBOR_OFFSETS.iter() {
            let neighbor_key = (key.0 + offset.0, key.1 + offset.1);
            if let Some(cells) = self.cells.get(&neighbor_key) {
                for cell in cells.iter() {
                    neighbors.push(*cell);
                }
            }
        }
        neighbors
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

#[derive(Component)]
struct SPHFluid {
    particles: Vec<Particle>,
    grid: SpatialGrid,
}

impl SPHFluid {
    fn new() -> Self {
        SPHFluid {
            particles: Vec::new(),
            grid: SpatialGrid::new(),
        }
    }

    fn init(&mut self) {
        self.particles = Vec::new();
        self.grid = SpatialGrid::new();
        let hx: f32 = PARTICLES_DX * NUM_PARTICLES_X as f32 / 2.0;
        let hy: f32 = PARTICLES_DY * NUM_PARTICLES_Y as f32 / 2.0;
        for i in 1..NUM_PARTICLES_X {
            for j in 1..NUM_PARTICLES_Y {
                let particle = Particle {
                    position: Vec3::new(
                        i as f32 * PARTICLES_DX - hx,
                        j as f32 * PARTICLES_DY - hy,
                        0.0,
                    ),
                    velocity: Vec3::ZERO,
                    mass: PARTICLE_MASS,
                    density: INITIAL_DENSITY,
                    pressure: Vec3::ZERO,
                    ext_force: Vec3::ZERO,
                };
                let cell = particle.to_cell();
                self.particles.push(particle);
                self.grid.insert(cell);
            }
        }
    }

    fn update_particle_forces(&mut self, dt: f32, simvars: &simui::FluidSimVars) {
        // should these be moved as struct vars?
        let threshold_radius = simvars.get("threshold_radius");
        let smoothing_radius = simvars.get("smoothing_radius");
        let viscosity = simvars.get("viscosity");
        let pressure_coeff = simvars.get("pressure");
        let wall_x = simvars.get("wall_x");
        let wall_y = simvars.get("wall_y");
        let restitution = simvars.get("restitution");
        let friction = simvars.get("friction");
        let gravity = simvars.get("gravity");
        let sim_speed = simvars.get("sim_speed");

        let dt = dt * sim_speed;
        for particle in self.particles.iter_mut() {
            let mut density = 0.0;
            let mut pressure_force = Vec3::ZERO;
            let mut viscosity_force = Vec3::ZERO;

            let neighbors = self.grid.get_neighbors(particle.position);
            for neighbor in neighbors.iter() {
                let r = particle.position - neighbor.position;
                let distance = r.length();
                if distance < threshold_radius {
                    density += neighbor.mass * poly6_kernel(distance, threshold_radius);
                    let grad_w = grad_poly6_kernel(r, threshold_radius);
                    // let lap_w = viscosity_kernel(r, *dist_threshold);
                    pressure_force +=
                        pressure_coeff * (neighbor.mass) * (grad_w / neighbor.mass.powi(2));
                    viscosity_force += viscosity
                        * viscosity_kernel(distance, smoothing_radius)
                        * (neighbor.velocity - particle.velocity);
                }
            }

            let acceleration = (pressure_force
                + viscosity_force
                + particle.ext_force
                + Vec3::new(0.0, -gravity, 0.0))
                / particle.mass;
            let velocity = particle.velocity + acceleration * dt;

            particle.density = density;
            particle.pressure = pressure_force;
            particle.velocity = velocity;
            particle.position += velocity * dt;

            let mut collision_normal = Vec3::ZERO;
            if particle.position.x < -wall_x {
                collision_normal = Vec3::X;
                particle.position.x = -wall_x;
            }
            if particle.position.x > wall_x {
                collision_normal = -Vec3::X;
                particle.position.x = wall_x;
            }
            if particle.position.y < -wall_y {
                collision_normal = Vec3::Y;
                particle.position.y = -wall_y;
            }
            if particle.position.y > wall_y {
                collision_normal = -Vec3::Y;
                particle.position.y = wall_y;
            }

            if collision_normal != Vec3::ZERO {
                let velocity_normal = particle.velocity.dot(collision_normal) * collision_normal;
                let velocity_tangential = particle.velocity - velocity_normal;

                particle.velocity =
                    -restitution * velocity_normal + (1.0 - friction) * velocity_tangential;
            }
        }
    }

    fn update_grid(&mut self) {
        self.grid.clear();
        for particle in self.particles.iter() {
            self.grid.insert(particle.to_cell());
        }
    }

    fn set_push_force(&mut self, point: Vec3, force: Vec3) {
        for particle in self.particles.iter_mut() {
            let distance: f32 = particle.position.distance(point);
            let logistic_response = 1.0 / (1.0 + f32::exp(1.0 + distance));
            particle.ext_force = force * logistic_response;
        }
    }

    fn set_grab_force(&mut self, point: Vec3, strength: f32, radius: f32, gravity: f32) {
        for particle in self.particles.iter_mut() {
            let vec = point - particle.position;
            let dist = vec.length();
            if dist < radius {
                let edge = (dist / radius);
                let center = 1.0 - edge;
                let direction = vec / dist;
                particle.ext_force = direction * center * strength + Vec3::new(0.0, gravity, 0.0);
            } else {
                particle.ext_force = Vec3::ZERO;
            }
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
    let mut fluid: SPHFluid = SPHFluid::new();
    let balls = fluid.get_balls();
    fluid.init();
    let simvars = simui::FluidSimVars {
        initialized: false,
        interact_mode: false,
        paused: false,
        debug: false,
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
    mut query: Query<(
        &mut SPHFluid,
        &mut Handle<MetaballMaterial>,
        &simui::FluidSimVars,
    )>,
    mut materials: ResMut<Assets<MetaballMaterial>>,
    mut gizmos: Gizmos,
) {
    let (mut fluid, mut handle, simvars) = query.single_mut();
    if !simvars.paused {
        let dt: f32 = time.delta_seconds();
        fluid.update_particle_forces(dt, simvars);
        fluid.update_grid();

        if let Some(material) = materials.get_mut(&*handle) {
            material.balls = fluid.get_balls();
        }
    }

    if (simvars.debug) {
        for particle in fluid.particles.iter() {
            gizmos.circle_2d(
                Vec2::new(particle.position.x, particle.position.y),
                1.,
                Color::rgba(1.0, 1.0, 1.0, 1.00),
            );
        }
    }
}

fn update_interactive(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mb: Res<ButtonInput<MouseButton>>,
    mut motion_er: EventReader<MouseMotion>,
    mut query: Query<(&mut SPHFluid, &simui::FluidSimVars)>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();
    let (mut fluid, simvars) = query.single_mut();
    fluid.set_push_force(Vec3::ZERO, Vec3::ZERO);

    let window: &Window = window_query.single();
    if let Some(cursor_position) = window.cursor_position() {
        if let Some(world_position) = camera.viewport_to_world_2d(camera_transform, cursor_position)
        {
            gizmos.circle_2d(world_position, 10., Color::WHITE);

            let point = Vec3::new(world_position.x, world_position.y, 0.0);

            let radius = simvars.get("interact_radius");
            let strength = simvars.get("interact_force");
            let gravity = simvars.get("gravity");
        if simvars.interact_mode {
                if mb.pressed(MouseButton::Left) {
                    println!("grab force {}", strength);
                    fluid.set_grab_force(point, strength, radius, gravity);
                }
            } else {
                let force_coeff = simvars.get("interact_force");
                for motion in motion_er.read() {
                    let force = Vec3::new(motion.delta.x, -motion.delta.y, 0.0);
                    fluid.set_push_force(point, force * force_coeff);
                }
            }
        }
    }
}

fn update_simvars(
    mut key_evr: EventReader<KeyboardInput>,
    mut fluidquery: Query<(&mut simui::FluidSimVars, &mut SPHFluid)>,
    query: Query<(&simui::SimVariable, &text_input::TextInputValue)>,
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
                fluid.init();
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

fn poly6_kernel(r: f32, h: f32) -> f32 {
    if r > h {
        0.0
    } else {
        let h2_r2 = h * h - r * r;
        h2_r2.powi(3)
    }
}

fn grad_poly6_kernel(r: Vec3, h: f32) -> Vec3 {
    let r_length = r.length();
    if r_length < h && r_length != 0.0 {
        let h2_r2 = h * h - r_length * r_length;
        return r * h2_r2.powi(2);
    }
    Vec3::ZERO
}

fn viscosity_kernel(r: f32, h: f32) -> f32 {
    if r < h {
        let h2_r2 = h * h - r * r;
        return h2_r2.powi(3);
    }
    0.0
}
