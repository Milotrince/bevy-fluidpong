// THIS IS A HACK PUT TOGETHER-- DO NOT WORK HERE.
// WORK ON JUST FLUID FIRST.
// - trinity

use bevy::{
    input::mouse::MouseMotion,
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
    window::PrimaryWindow,
};
use std::{f32::consts::PI, path};

const BALL_SPEED: f32 = 5.;
const BALL_SIZE: f32 = 5.;
const PADDLE_SPEED: f32 = 4.;
const PADDLE_WIDTH: f32 = 10.;
const PADDLE_HEIGHT: f32 = 50.;
const GUTTER_HEIGHT: f32 = 96.;

#[derive(Component)]
struct Player1Score;

#[derive(Component)]
struct Player2Score;

#[derive(Resource, Default)]
struct Score {
    player1: u32,
    player2: u32,
}

enum Scorer {
    Player1,
    Player2,
}

#[derive(Event)]
struct Scored(Scorer);

#[derive(Component)]
struct Ball;

#[derive(Bundle)]
struct BallBundle {
    ball: Ball,
    shape: Shape,
    velocity: Velocity,
    position: Position,
}

impl BallBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            ball: Ball,
            shape: Shape(Vec2::splat(BALL_SIZE)),
            velocity: Velocity(Vec2::new(x, y)),
            position: Position(Vec2::new(0., 0.)),
        }
    }
}

#[derive(Component)]
struct Paddle;

#[derive(Bundle)]
struct PaddleBundle {
    paddle: Paddle,
    shape: Shape,
    position: Position,
    velocity: Velocity,
}

impl PaddleBundle {
    fn new(x: f32, y: f32) -> Self {
        Self {
            paddle: Paddle,
            shape: Shape(Vec2::new(PADDLE_WIDTH, PADDLE_HEIGHT)),
            position: Position(Vec2::new(x, y)),
            velocity: Velocity(Vec2::new(0., 0.)),
        }
    }
}

#[derive(Component)]
struct Gutter;

#[derive(Bundle)]
struct GutterBundle {
    gutter: Gutter,
    shape: Shape,
    position: Position,
}

impl GutterBundle {
    fn new(x: f32, y: f32, w: f32) -> Self {
        Self {
            gutter: Gutter,
            shape: Shape(Vec2::new(w, GUTTER_HEIGHT)),
            position: Position(Vec2::new(x, y)),
        }
    }
}

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Shape(Vec2);

#[derive(Component)]
struct Player1;

#[derive(Component)]
struct Player2;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

pub struct FluidPongPlugin;

impl Plugin for FluidPongPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_plugins(Material2dPlugin::<MetaballMaterial>::default())
            .add_event::<Scored>()
            .add_systems(
                Startup,
                (
                    spawn_ball,
                    spawn_paddles,
                    spawn_gutters,
                    spawn_scoreboard,
                    init_fluid.before(spawn_paddles),
                ),
            )
            .add_systems(
                Update,
                (
                    move_ball,
                    handle_player_input,
                    detect_scoring,
                    reset_ball.after(detect_scoring),
                    update_score.after(detect_scoring),
                    update_scoreboard.after(update_score),
                    move_paddles.after(handle_player_input),
                    project_positions.after(move_ball),
                    handle_collisions.after(move_ball),
                    update_interactive,
                    update_fluid.after(update_interactive),
                ),
            );
    }
}

fn update_scoreboard(
    mut player1_score: Query<&mut Text, With<Player1Score>>,
    mut player2_score: Query<&mut Text, (With<Player2Score>, Without<Player1Score>)>,
    score: Res<Score>,
) {
    if score.is_changed() {
        if let Ok(mut player1_score) = player1_score.get_single_mut() {
            player1_score.sections[0].value = score.player1.to_string();
        }

        if let Ok(mut player2_score) = player2_score.get_single_mut() {
            player2_score.sections[0].value = score.player2.to_string();
        }
    }
}

fn spawn_scoreboard(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "0",
            TextStyle {
                font_size: 72.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(15.0),
            ..default()
        }),
        Player1Score,
    ));

    commands.spawn((
        TextBundle::from_section(
            "0",
            TextStyle {
                font_size: 72.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(15.0),
            ..default()
        }),
        Player2Score,
    ));
}

fn update_score(mut score: ResMut<Score>, mut events: EventReader<Scored>) {
    for event in events.read() {
        match event.0 {
            Scorer::Player1 => score.player1 += 1,
            Scorer::Player2 => score.player2 += 1,
        }
    }
}

fn detect_scoring(
    mut ball: Query<&mut Position, With<Ball>>,
    window: Query<&Window>,
    mut events: EventWriter<Scored>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();

        if let Ok(ball) = ball.get_single_mut() {
            if ball.0.x > window_width / 2. {
                events.send(Scored(Scorer::Player2));
            } else if ball.0.x < -window_width / 2. {
                events.send(Scored(Scorer::Player1));
            }
        }
    }
}

fn reset_ball(
    mut ball: Query<(&mut Position, &mut Velocity), With<Ball>>,
    mut events: EventReader<Scored>,
    mut score: ResMut<Score>
) {
    for event in events.read() {
        if let Ok((mut position, mut velocity)) = ball.get_single_mut() {
            match event.0 {
                Scorer::Player2 => {
                    position.0 = Vec2::new(0., 0.);
                    velocity.0 = Vec2::new(-1., if score.player2 % 2 == 0 { 1.0 } else { -1.0 });
                }
                Scorer::Player1 => {
                    position.0 = Vec2::new(0., 0.);
                    velocity.0 = Vec2::new(1., if score.player1 % 2 == 0 { 1.0 } else { -1.0 });
                }
            }
        }
    }
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle1: Query<&mut Velocity, With<Player1>>,
    mut paddle2: Query<&mut Velocity, (With<Player2>, Without<Player1>)>,
) {
    if let Ok(mut velocity) = paddle1.get_single_mut() {
        if keyboard_input.pressed(KeyCode::KeyW) {
            velocity.0.y = 1.;
        } else if keyboard_input.pressed(KeyCode::KeyS) {
            velocity.0.y = -1.;
        } else {
            velocity.0.y = 0.;
        }
    }
    if let Ok(mut velocity) = paddle2.get_single_mut() {
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            velocity.0.y = 1.;
        } else if keyboard_input.pressed(KeyCode::ArrowDown) {
            velocity.0.y = -1.;
        } else {
            velocity.0.y = 0.;
        }
    }
}

fn spawn_gutters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let window_height = window.resolution.height();

        let top_gutter_y = window_height / 2. - GUTTER_HEIGHT / 2.;
        let bottom_gutter_y = -window_height / 2. + GUTTER_HEIGHT / 2.;

        let top_gutter = GutterBundle::new(0., top_gutter_y, window_width);
        let bottom_gutter = GutterBundle::new(0., bottom_gutter_y, window_width);

        let mesh = Mesh::from(Rectangle::from_size(top_gutter.shape.0));
        let material = ColorMaterial::from(Color::rgb(0., 0., 0.));

        let mesh_handle = meshes.add(mesh);
        let material_handle = materials.add(material);

        commands.spawn((
            top_gutter,
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: material_handle.clone(),
                ..default()
            },
        ));

        commands.spawn((
            bottom_gutter,
            MaterialMesh2dBundle {
                mesh: mesh_handle.into(),
                material: material_handle.clone(),
                ..default()
            },
        ));
    }
}

fn project_positions(mut ball: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut ball {
        transform.translation = position.0.extend(0.);
    }
}

fn move_ball(mut ball: Query<(&mut Position, &Velocity), With<Ball>>) {
    if let Ok((mut position, velocity)) = ball.get_single_mut() {
        position.0 += velocity.0 * BALL_SPEED;
    }
}

fn move_paddles(
    mut paddle: Query<(&mut Position, &Velocity), With<Paddle>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_height = window.resolution.height();

        for (mut position, velocity) in &mut paddle {
            let new_position = position.0 + velocity.0 * PADDLE_SPEED;
            if new_position.y.abs() < window_height / 2. - GUTTER_HEIGHT - PADDLE_HEIGHT / 2. {
                position.0 = new_position;
            }
        }
    }
}

fn collide_with_side(ball: BoundingCircle, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest = wall.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn handle_collisions(
    mut ball: Query<(&mut Velocity, &Position, &Shape), With<Ball>>,
    other_things: Query<(&Position, &Shape), Without<Ball>>,
) {
    if let Ok((mut ball_velocity, ball_position, ball_shape)) = ball.get_single_mut() {
        for (position, shape) in &other_things {
            if let Some(collision) = collide_with_side(
                BoundingCircle::new(ball_position.0, ball_shape.0.x),
                Aabb2d::new(position.0, shape.0 / 2.),
            ) {
                match collision {
                    Collision::Left => {
                        ball_velocity.0.x *= -1.;
                    }
                    Collision::Right => {
                        ball_velocity.0.x *= -1.;
                    }
                    Collision::Top => {
                        ball_velocity.0.y *= -1.;
                    }
                    Collision::Bottom => {
                        ball_velocity.0.y *= -1.;
                    }
                }
            }
        }
    }
}

fn spawn_paddles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let window_width = window.resolution.width();
        let padding = 50.;
        let right_paddle_x = window_width / 2. - padding;
        let left_paddle_x = -window_width / 2. + padding;

        let mesh = Mesh::from(Rectangle::new(PADDLE_WIDTH, PADDLE_HEIGHT));

        let mesh_handle = meshes.add(mesh);

        commands.spawn((
            Player1,
            PaddleBundle::new(left_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                ..default()
            },
        ));

        commands.spawn((
            Player2,
            PaddleBundle::new(right_paddle_x, 0.),
            MaterialMesh2dBundle {
                mesh: mesh_handle.clone().into(),
                material: materials.add(ColorMaterial::from(Color::WHITE)),
                ..default()
            },
        ));
    }
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let shape = Mesh::from(Circle::new(BALL_SIZE));
    let color = ColorMaterial::from(Color::WHITE);

    let mesh_handle = meshes.add(shape);
    let material_handle = materials.add(color);

    commands.spawn((
        BallBundle::new(1., 1.),
        MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material: material_handle,
            ..default()
        },
    ));
}

// ----- FLUID

const INITIAL_DENSITY: f32 = 100.0;
const SMOOTHING_LENGTH: f32 = 4.0;
const VISCOSITY_COEFFICIENT: f32 = 0.0001;
const INTERACT_FORCE: f32 = 2000.0;
const INTERACT_RADIUS: f32 = 6.0;
// const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

const RESTITUTION_COEFFICIENT: f32 = 0.2;
const FRICTION_COEFFICIENT: f32 = 0.7;

const PARTICLE_SIZE: f32 = 2.0;
const NUM_PARTICLES_X: u32 = 64; //64;
const NUM_PARTICLES_Y: u32 = 64; //64;
const NUM_PARTICLES: usize = (NUM_PARTICLES_X * NUM_PARTICLES_Y) as usize;

const PARTICLES_DX: f32 = 8.0;
const PARTICLES_DY: f32 = 4.0;
const PARTICLE_MASS: f32 = 10.0;
const GRID_CELL_SIZE: f32 = 10.0;

// use smaller when testing
const WALL_X_MIN: f32 = -320.0;
const WALL_X_MAX: f32 = 320.0;
const WALL_Y_MIN: f32 = -240.0 + GUTTER_HEIGHT;
const WALL_Y_MAX: f32 = 240.0 - GUTTER_HEIGHT;

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
            mass: self.mass,
        };
    }
}

#[derive(Copy, Clone)]
struct Cell {
    position: Vec3,
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

            let acceleration = (pressure + force + particle.ext_force) / particle.mass;
            let velocity = particle.velocity + acceleration * dt;

            particle.density = density;
            particle.pressure = pressure;
            particle.velocity = velocity;
            particle.position += velocity * dt;

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
        }
    }

    fn update_grid(&mut self) {
        self.grid.clear();
        for particle in self.particles.iter() {
            self.grid.insert(particle.to_cell());
        }
    }

    fn set_external_force(&mut self, point: Vec3, force: Vec3) {
        let scale = 1.0 / INTERACT_RADIUS;
        for particle in self.particles.iter_mut() {
            let distance: f32 = particle.position.distance(point);
            let logistic_response = 1.0 / (1.0 + f32::exp(-scale * (INTERACT_RADIUS - distance)));
            particle.ext_force = force * INTERACT_FORCE * logistic_response;
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
    // fn vertex_shader() -> ShaderRef {
    //     "shaders/metaball.wgsl".into()
    // }
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

    commands.spawn((
        fluid,
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(
                WALL_X_MAX - WALL_X_MIN,
                WALL_Y_MAX - WALL_Y_MIN,
            ))),
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
    mut query: Query<(&mut SPHFluid, &mut Handle<MetaballMaterial>)>,
    mut materials: ResMut<Assets<MetaballMaterial>>,
    mut gizmos: Gizmos,
) {
    let dt: f32 = time.delta_seconds();
    let (mut fluid, mut handle) = query.single_mut();
    fluid.update_particle_forces(dt);
    fluid.update_grid();

    if let Some(material) = materials.get_mut(&*handle) {
        material.balls = fluid.get_balls();
    }
}

fn update_interactive(
    mut ball: Query<(&mut Position, &Velocity), With<Ball>>,
    mut fluid_query: Query<&mut SPHFluid>,
) {
    let mut fluid = fluid_query.single_mut();
    if let Ok((mut position, velocity)) = ball.get_single_mut() {
        position.0 += velocity.0 * BALL_SPEED;
        let point = Vec3::new(position.0.x, position.0.y, 0.0);
        let force = Vec3::new(velocity.0.x, velocity.0.y, 0.0);

        fluid.set_external_force(point, force);
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
