use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::PrimaryWindow,
};
use std::{f32::consts::PI, path};

const INITIAL_DENSITY: f32 = 0.5;
const SMOOTHING_LENGTH: f32 = 0.5;
const VISCOSITY_COEFFICIENT: f32 = 10.0;
const INTERACT_FORCE: f32 = 5.0;
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
                update_positions,
                update_spatial_grid,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
struct Particle {
    position: Vec3,
    velocity: Vec3,
    mass: f32,
    density: f32,
    pressure: Vec3,
    force: Vec3,
    color: Color,
}

#[derive(Resource)]
pub struct SpatialGrid {
    cells: HashMap<(i32, i32, i32), Vec<Entity>>,
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

    pub fn insert(&mut self, entity: Entity, position: Vec3) {
        let key = (
            (position.x / GRID_CELL_SIZE).floor() as i32,
            (position.y / GRID_CELL_SIZE).floor() as i32,
            (position.z / GRID_CELL_SIZE).floor() as i32,
        );
        self.cells.entry(key).or_insert_with(Vec::new).push(entity);
    }

    pub fn get_neighbors(&self, position: Vec3) -> Vec<Entity> {
        let mut neighbors = Vec::new();
        let key = (
            (position.x / GRID_CELL_SIZE).floor() as i32,
            (position.y / GRID_CELL_SIZE).floor() as i32,
            (position.z / GRID_CELL_SIZE).floor() as i32,
        );
        for offset in Self::NEIGHBOR_OFFSETS.iter() {
            let neighbor_key = (key.0 + offset.0, key.1 + offset.1, key.2 + offset.2);
            if let Some(entities) = self.cells.get(&neighbor_key) {
                neighbors.extend(entities.iter().copied());
            }
        }
        neighbors
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }
}

fn init_fluid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut grid: ResMut<SpatialGrid>,
) {
    commands.spawn(Camera2dBundle::default());

    for i in 1..NUM_PARTICLES_X {
        for j in 1..NUM_PARTICLES_Y {
            let color: Color = Color::rgb(
                i as f32 / NUM_PARTICLES_X as f32,
                0.0,
                j as f32 / NUM_PARTICLES_Y as f32,
            );
            let position: Vec3 = Vec3::new(i as f32 * PARTICLES_DX, j as f32 * PARTICLES_DY, 0.0);
            let entity = commands
                .spawn((
                    Particle {
                        position: position,
                        velocity: Vec3::new(0.0, 0.0, 0.0),
                        mass: PARTICLE_MASS,
                        density: INITIAL_DENSITY,
                        pressure: Vec3::new(0.0, 0.0, 0.0),
                        force: Vec3::new(0.0, 0.0, 0.0),
                        color: color,
                    },
                    MaterialMesh2dBundle {
                        mesh: Mesh2dHandle(meshes.add(Circle { radius: PARTICLE_SIZE })),
                        material: materials.add(color),
                        transform: Transform::from_translation(position),
                        ..default()
                    },
                ))
                .id();
            grid.insert(entity, position)
        }
    }
}

fn update_fluid(
    time: Res<Time>,
    grid: Res<SpatialGrid>,
    mut query: Query<(Entity, &mut Particle)>,
) {
    let dt = time.delta_seconds();
    let particle_data: Vec<(Entity, Vec3, f32)> = query
        .iter_mut()
        .map(|(e, p)| (e, p.position, p.mass))
        .collect();

    let mut density_updates = Vec::new();
    let mut force_updates = Vec::new();

    for (entity, position, _) in particle_data.iter() {
        let mut density = 0.0;
        let mut pressure = Vec3::ZERO;
        let mut force = Vec3::ZERO;

        let neighbors = grid.get_neighbors(*position);
        for neighbor_entity in &neighbors {
            if let Some((_, neighbor_position, neighbor_mass)) =
                particle_data.iter().find(|(e, _, _)| e == neighbor_entity)
            {
                let r = *position - *neighbor_position;
                let distance = r.length();
                if 0.0 < distance && distance < SMOOTHING_LENGTH {
                    density += neighbor_mass * poly6_kernel(distance, SMOOTHING_LENGTH);
                    let grad_w = grad_poly6_kernel(r, SMOOTHING_LENGTH);
                    let lap_w = laplacian_viscosity_kernel(r, SMOOTHING_LENGTH);
                    pressure += -(neighbor_mass) * (grad_w / neighbor_mass.powi(2));
                    force += VISCOSITY_COEFFICIENT * (neighbor_mass) * (lap_w);
                }
            }
        }

        density_updates.push((entity, density));
        force_updates.push((entity, pressure, force));
    }

    for (entity, new_density) in density_updates {
        if let Ok((_, mut particle)) = query.get_mut(*entity) {
            particle.density = new_density;
        }
    }

    for (entity, new_pressure, new_force) in force_updates {
        if let Ok((_, mut particle)) = query.get_mut(*entity) {
            particle.pressure = new_pressure;
            particle.force = new_force;

            let acceleration = (new_pressure + new_force) / particle.mass;
            particle.velocity += acceleration * dt;

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
    
                particle.velocity = -RESTITUTION_COEFFICIENT * velocity_normal + (1.0 - FRICTION_COEFFICIENT) * velocity_tangential;
            }

            let vel = particle.velocity;
            particle.position += vel * dt;

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

fn update_positions(mut query: Query<(&mut Transform, &Particle)>) {
    for (mut transform, particle) in query.iter_mut() {
        transform.translation = particle.position;
    }
}

fn update_interactive(
    time: Res<Time>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut motion_er: EventReader<MouseMotion>,
    grid: Res<SpatialGrid>,
    mut query: Query<&mut Particle>,
    mut gizmos: Gizmos,
) {
    let dt = time.delta_seconds();
    let (camera, camera_transform) = camera_query.single();

    for motion in motion_er.read() {
        let window: &Window = window_query.single();
        if let Some(cursor_position) = window.cursor_position() {
            if let Some(world_position2) =
                camera.viewport_to_world_2d(camera_transform, cursor_position)
            {
                // println!(
                //     "Mouse moved: X: {} px, Y: {} px",
                //     motion.delta.x, motion.delta.y
                // );
                let world_position = Vec3::new(world_position2.x, world_position2.y, 0.0);
                gizmos.circle_2d(world_position2, 10., Color::WHITE);

                // affect nearby particles
                let neighbor_entities: Vec<Entity> = grid.get_neighbors(world_position);
                for neighbor_entity in neighbor_entities {
                    if let Ok(mut particle) = query.get_mut(neighbor_entity) {
                        let direction = (particle.position - world_position).normalize();
                        let force = direction * INTERACT_FORCE;
                        particle.velocity += force;
                        let vel = particle.velocity;
                        particle.position += vel * dt;
                    }
                }
            }
        }
    }
}

fn update_spatial_grid(mut grid: ResMut<SpatialGrid>, query: Query<(Entity, &Transform)>) {
    grid.clear();

    for (entity, transform) in query.iter() {
        grid.insert(entity, transform.translation);
    }
}
