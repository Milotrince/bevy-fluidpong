use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::PrimaryWindow,
};
use std::{f32::consts::PI, path};

const INITIAL_DENSITY: f32 = 0.5;
const SMOOTHING_LENGTH: f32 = 0.5;
const VISCOSITY_COEFFICIENT: f32 = 10.0;
const INTERACT_FORCE: f32 = 5.0;
const INTERACT_RADIUS: f32 = 2.0;
const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

const RESTITUTION_COEFFICIENT: f32 = 0.8;
const FRICTION_COEFFICIENT: f32 = 0.5;

const PARTICLE_SIZE: f32 = 4.0;
const NUM_PARTICLES_X: i32 = 10;
const NUM_PARTICLES_Y: i32 = 10;
const PARTICLES_DX: f32 = 4.0;
const PARTICLES_DY: f32 = 4.0;
const PARTICLE_MASS: f32 = 100.0;
const GRID_CELL_SIZE: f32 = 1000.0;

const WALL_X_MIN: f32 = -100.0;
const WALL_X_MAX: f32 = 100.0;
const WALL_Y_MIN: f32 = -100.0;
const WALL_Y_MAX: f32 = 100.0;

pub struct FluidPlugin;

impl Plugin for FluidPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_fluid).add_systems(
            Update,
            (
                update_fluid,
                update_interactive,
            )
                .chain(),
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
        return Cell { position: self.position, mass: self.mass };
    }
}

#[derive(Copy, Clone)]
struct Cell {
    position: Vec3,
    mass: f32
}

pub struct SpatialGrid {
    cells: HashMap<(i32, i32, i32), Vec<Cell>>,
}

impl SpatialGrid {
    const NEIGHBOR_OFFSETS: [(i32, i32, i32); 7] = [
        (0, 0, 0),
        (-1, 0, 0),
        (1, 0, 0),
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
    ];

    pub fn new() -> Self {
        SpatialGrid {
            cells: HashMap::new(),
        }
    }

    pub fn insert(&mut self, cell: Cell) {
        let key = (
            (cell.position.x / GRID_CELL_SIZE).floor() as i32,
            (cell.position.y / GRID_CELL_SIZE).floor() as i32,
            (cell.position.z / GRID_CELL_SIZE).floor() as i32,
        );
        self.cells
            .entry(key)
            .or_insert_with(Vec::new)
            .push(cell);
    }

    pub fn get_neighbors(&self, position: Vec3) -> Vec<Cell> {
        let mut neighbors = Vec::new();
        let key = (
            (position.x / GRID_CELL_SIZE).floor() as i32,
            (position.y / GRID_CELL_SIZE).floor() as i32,
            (position.z / GRID_CELL_SIZE).floor() as i32,
        );
        for offset in Self::NEIGHBOR_OFFSETS.iter() {
            let neighbor_key = (key.0 + offset.0, key.1 + offset.1, key.2 + offset.2);
            if let Some(cells) = self.cells.get(&neighbor_key) {
                for cell in cells.iter() {
                    neighbors.push(*cell);
                }
            }
        }
        neighbors
    }

    // fn get_balls(self) -> Vec<Vec3> {
    //     self.particles.iter().map(|p| p.position).collect()
    // }

    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

#[derive(Component)]
struct Fluid {
    particles: Vec<Particle>,
    grid: SpatialGrid,
}

impl Fluid {
    fn new() -> Self {
        Fluid {
            particles: Vec::new(),
            grid: SpatialGrid::new(),
        }
    }

    fn init(&mut self) {
        for i in 1..NUM_PARTICLES_X {
            for j in 1..NUM_PARTICLES_Y {
                // let color: Color = Color::rgb(
                //     i as f32 / NUM_PARTICLES_X as f32,
                //     0.0,
                //     j as f32 / NUM_PARTICLES_Y as f32,
                // );
                let particle = Particle {
                    position: Vec3::new(i as f32 * PARTICLES_DX, j as f32 * PARTICLES_DY, 0.0),
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

    fn update_particle_forces(&mut self, dt: f32) {
        for particle in self.particles.iter_mut() {
            let mut density = 0.0;
            let mut pressure = Vec3::ZERO;
            let mut force = Vec3::ZERO;

            let neighbors = self.grid.get_neighbors(particle.position);
            for neighbor in neighbors.iter() {
                let r = particle.position - neighbor.position;
                let distance = r.length();
                if 0.0 < distance && distance < SMOOTHING_LENGTH {
                    density += neighbor.mass * poly6_kernel(distance, SMOOTHING_LENGTH);
                    let grad_w = grad_poly6_kernel(r, SMOOTHING_LENGTH);
                    let lap_w = laplacian_viscosity_kernel(r, SMOOTHING_LENGTH);
                    pressure += -(neighbor.mass) * (grad_w / neighbor.mass.powi(2));
                    force += VISCOSITY_COEFFICIENT * (neighbor.mass) * (lap_w);
                }
            }

            let mut collision_normal = Vec3::ZERO;
            if particle.position.x < WALL_X_MIN {
                collision_normal = Vec3::X;
                particle.position.x = WALL_X_MIN;
            }
            if particle.position.x > WALL_X_MAX {
                collision_normal = -Vec3::X;
                particle.position.x = WALL_X_MAX;
            }
            if particle.position.y < WALL_Y_MIN {
                collision_normal = Vec3::Y;
                particle.position.y = WALL_Y_MIN;
            }
            if particle.position.y > WALL_Y_MAX {
                collision_normal = -Vec3::Y;
                particle.position.y = WALL_Y_MAX;
            }

            if collision_normal != Vec3::ZERO {
                let velocity_normal = particle.velocity.dot(collision_normal) * collision_normal;
                let velocity_tangential = particle.velocity - velocity_normal;

                particle.velocity = -RESTITUTION_COEFFICIENT * velocity_normal
                    + (1.0 - FRICTION_COEFFICIENT) * velocity_tangential;
            }

            let acceleration = (pressure + force + particle.ext_force) / particle.mass;
            let velocity = particle.velocity + acceleration * dt;

            particle.density = density;
            particle.pressure = pressure;
            particle.velocity = velocity;
            particle.position += velocity * dt;
        }
    }

    fn update_grid(&mut self) {
        self.grid.clear();
        for particle in self.particles.iter() {
            self.grid.insert(particle.to_cell());
        }
    }

    fn set_external_force(&mut self, point: Vec3, force: Vec3) {
        for particle in self.particles.iter_mut() {
            let distance: f32 = particle.position.distance(point);
            particle.ext_force = force / distance.powi(2)
        }
    }

    fn get_balls(&self) -> Vec<Vec3> {
        self.particles.iter().map(|p| p.position).collect()
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct FluidMaterial {
    // #[uniform(0)]
    color: Color,
    // #[storage(1)]
    balls: Vec<Vec3>,
    // #[uniform(2)]
    radius: f32,
}

impl Material2d for FluidMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/metaball.wgsl".into()
    }
}

fn init_fluid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FluidMaterial>>,
) {
    let mut fluid: Fluid = Fluid::new();
    let balls = fluid.get_balls();
    fluid.init();

    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        fluid,
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(
                WALL_X_MAX - WALL_X_MIN,
                WALL_Y_MAX - WALL_Y_MIN,
            ))),
            material: materials.add(FluidMaterial {
                color: Color::ALICE_BLUE,
                balls: balls,
                radius: PARTICLE_SIZE,
            }),
            transform: Transform::from_translation(Vec3::ZERO),
            ..default()
        },
    ));
}

fn update_fluid(time: Res<Time>, mut query: Query<(&mut Fluid, &mut Handle<FluidMaterial>)>, mut materials: ResMut<Assets<FluidMaterial>>) {
    let dt: f32 = time.delta_seconds();
    let (mut fluid, mut handle)  = query.single_mut();
    fluid.update_particle_forces(dt);
    fluid.update_grid();

    let material = materials.get_mut(&*handle);
    if let Some(mut material) = materials.get_mut(&*handle) {
        material.balls = fluid.get_balls();
    }
}


fn update_interactive(
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut motion_er: EventReader<MouseMotion>,
    mut query: Query<&mut Fluid>,
    mut gizmos: Gizmos,
) {
    let (camera, camera_transform) = camera_query.single();

    for motion in motion_er.read() {
        let window: &Window = window_query.single();
        if let Some(cursor_position) = window.cursor_position() {
            if let Some(world_position) =
                camera.viewport_to_world_2d(camera_transform, cursor_position)
            {
                gizmos.circle_2d(world_position, 10., Color::WHITE);

                let point = Vec3::new(world_position.x, world_position.y, 0.0);
                let force = Vec3::new(motion.delta.x, motion.delta.y, 0.0);

                let mut fluid = query.single_mut();
                fluid.set_external_force(point, force * INTERACT_FORCE);
            }
        }
    }
}



fn poly6_kernel(r: f32, h: f32) -> f32 {
    if r > h {
        0.0
    } else {
        let coefficient = 315.0 / (64.0 * PI * h.powi(9));
        let h2_r2 = h * h - r * r;
        coefficient * h2_r2.powi(3)
    }
}

fn grad_poly6_kernel(r: Vec3, h: f32) -> Vec3 {
    let r_length = r.length();
    if r_length < h && r_length != 0.0 {
        let coefficient = -945.0 / (32.0 * std::f32::consts::PI * h.powi(9));
        let h2_r2 = h * h - r_length * r_length;
        let factor = coefficient * h2_r2.powi(2);
        return r * factor;
    }
    Vec3::ZERO
}

fn laplacian_viscosity_kernel(r: Vec3, h: f32) -> f32 {
    let r_length = r.length();
    if r_length < h {
        let coefficient = 45.0 / (std::f32::consts::PI * h.powi(6));
        return coefficient * (h - r_length);
    }
    0.0
}
