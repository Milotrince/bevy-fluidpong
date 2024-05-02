pub mod pongfluid;

use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use pongfluid::PongFluid;

use crate::{GAME_HEIGHT, GAME_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH};

const BALL_INITIAL_SPEED: f32 = 3.;
const BALL_SIZE: f32 = 5.;
const PADDLE_SPEED: f32 = 6.;
const PADDLE_WIDTH: f32 = 10.;
const PADDLE_HEIGHT: f32 = 50.;
const GUTTER_HEIGHT: f32 = (SCREEN_HEIGHT - GAME_HEIGHT) / 2.0;

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
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Shape(Vec2);

#[derive(Component)]
struct Player1;

#[derive(Component)]
struct Player2;

pub struct PongPlugin;

impl Plugin for PongPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_event::<Scored>()
            .add_systems(
                Startup,
                (configure_gizmos, spawn_ball, spawn_paddles, spawn_scoreboard).chain(),
            )
            .add_systems(
                Update,
                (
                    draw_gizmos,
                    move_ball.before(move_paddles),
                    handle_player_input,
                    detect_scoring,
                    reset_ball.after(detect_scoring),
                    update_score.after(detect_scoring),
                    update_scoreboard.after(update_score),
                    move_paddles.after(handle_player_input),
                    project_positions.after(move_ball),
                    handle_collisions.after(move_ball),
                    handle_player_input_fluid.after(move_ball),
                ),
            );
    }
}

fn configure_gizmos(mut config_store: ResMut<GizmoConfigStore>) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.line_width = 10.0;
}

fn draw_dotted_line(
    gizmos: &mut Gizmos,
    start: Vec2,
    end: Vec2,
    color: Color,
    segment_length: f32,
    gap_length: f32,
) {
    let direction = end - start;
    let total_length = direction.length();
    let mut current_length = 0.0;

    while current_length < total_length {
        let segment_end = current_length + segment_length;
        let end_point = if segment_end < total_length {
            start + direction.normalize() * segment_end
        } else {
            end
        };
        gizmos.line_2d(start + direction.normalize() * current_length, end_point, color);
        current_length += segment_length + gap_length;
    }
}

fn draw_gizmos(mut gizmos: Gizmos) {
    let segment_length = 5.0;
    let gap_length = 2.5;

    draw_dotted_line(
        &mut gizmos,
        Vec2::new(0.0, GAME_HEIGHT / 2.0),
        Vec2::new(0.0, -GAME_HEIGHT / 2.0),
        Color::WHITE,
        segment_length,
        gap_length,
    );
    draw_dotted_line(
        &mut gizmos,
        Vec2::new(-GAME_WIDTH / 2.0, GAME_HEIGHT / 2.0),
        Vec2::new(GAME_WIDTH / 2.0, GAME_HEIGHT / 2.0),
        Color::WHITE,
        segment_length,
        gap_length,
    );
    draw_dotted_line(
        &mut gizmos,
        Vec2::new(-GAME_WIDTH / 2.0, -GAME_HEIGHT / 2.0),
        Vec2::new(GAME_WIDTH / 2.0, -GAME_HEIGHT / 2.0),
        Color::WHITE,
        segment_length,
        gap_length,
    );
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
            TextStyle { font_size: 72.0, color: Color::WHITE, ..default() },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            right: Val::Px(SCREEN_WIDTH / 2.0 * 0.8),
            ..default()
        }),
        Player1Score,
    ));

    commands.spawn((
        TextBundle::from_section(
            "0",
            TextStyle { font_size: 72.0, color: Color::WHITE, ..default() },
        )
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(SCREEN_WIDTH / 2.0 * 0.8),
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

fn detect_scoring(mut ball: Query<&mut Position, With<Ball>>, mut events: EventWriter<Scored>) {
    if let Ok(ball) = ball.get_single_mut() {
        if ball.0.x > GAME_WIDTH / 2. {
            events.send(Scored(Scorer::Player2));
        } else if ball.0.x < -GAME_WIDTH / 2. {
            events.send(Scored(Scorer::Player1));
        }
    }
}

fn reset_ball(
    mut ball: Query<(&mut Position, &mut Velocity), With<Ball>>,
    mut events: EventReader<Scored>,
) {
    for event in events.read() {
        if let Ok((mut position, mut velocity)) = ball.get_single_mut() {
            match event.0 {
                Scorer::Player2 => {
                    position.0 = Vec2::new(0., 0.);
                    velocity.0 = Vec2::new(-1., 1.) * BALL_INITIAL_SPEED;
                }
                Scorer::Player1 => {
                    position.0 = Vec2::new(0., 0.);
                    velocity.0 = Vec2::new(1., 1.) * BALL_INITIAL_SPEED;
                }
            }
        }
    }
}

fn handle_player_input_fluid(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut sphfluid_query: Query<&mut crate::sph::fluid::Fluid>,
    mut nsfluid_query: Query<&mut crate::ns::fluid::Fluid>,
    mut paddle1: Query<&Position, With<Player1>>,
    mut paddle2: Query<&Position, (With<Player2>, Without<Player1>)>,
) {
    if let Ok(mut fluid) = sphfluid_query.get_single_mut() {
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            if let Ok(position) = paddle1.get_single_mut() {
                fluid.apply_emit_force(position.0, Vec2::new(1.0, 0.0))
            }
        }
        if keyboard_input.pressed(KeyCode::ShiftRight) {
            if let Ok(position) = paddle2.get_single_mut() {
                fluid.apply_emit_force(position.0, Vec2::new(-1.0, 0.0))
            }
        }
    }
    if let Ok(mut fluid) = nsfluid_query.get_single_mut() {
        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            if let Ok(position) = paddle1.get_single_mut() {
                fluid.apply_emit_force(position.0, Vec2::new(1.0, 0.0))
            }
        }
        if keyboard_input.pressed(KeyCode::ShiftRight) {
            if let Ok(position) = paddle2.get_single_mut() {
                fluid.apply_emit_force(position.0, Vec2::new(-1.0, 0.0))
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

fn project_positions(mut ball: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut ball {
        transform.translation = position.0.extend(0.);
    }
}

fn move_ball(
    mut ball: Query<(&mut Position, &mut Velocity), With<Ball>>,
    mut sphfluid_query: Query<&mut crate::sph::fluid::Fluid>,
    mut nsfluid_query: Query<&mut crate::ns::fluid::Fluid>,
) {
    if let Ok((mut position, mut velocity)) = ball.get_single_mut() {
        let vel = velocity.0;
        let pos = position.0;
        if let Ok(mut fluid) = sphfluid_query.get_single_mut() {
            velocity.0 += fluid.get_fluid_force_at(pos, vel);
            fluid.apply_ball_force(pos, vel);
        }
        if let Ok(mut fluid) = nsfluid_query.get_single_mut() {
            velocity.0 += fluid.get_fluid_force_at(pos, vel);
            fluid.apply_ball_force(pos, vel);
        }
    }
}

fn move_paddles(
    mut paddle: Query<(&mut Position, &Velocity), With<Paddle>>,
    mut sphfluid_query: Query<&mut crate::sph::fluid::Fluid>,
    mut nsfluid_query: Query<&mut crate::ns::fluid::Fluid>,
) {
    for (mut position, velocity) in &mut paddle {
        let vel = velocity.0 * PADDLE_SPEED;
        let new_position = position.0 + vel;
        if new_position.y.abs() < GAME_HEIGHT / 2. - PADDLE_HEIGHT / 2. {
            position.0 = new_position;
            if let Ok(mut fluid) = sphfluid_query.get_single_mut() {
                fluid.apply_paddle_force(position.0, vel);
            }
            if let Ok(mut fluid) = nsfluid_query.get_single_mut() {
                fluid.apply_paddle_force(position.0, vel);
            }
        }
    }
}

fn handle_collisions(
    mut ball: Query<(&mut Velocity, &mut Position, &Shape), With<Ball>>,
    paddles: Query<(&Position, &Shape), (With<Paddle>, Without<Ball>)>,
) {
    if let Ok((mut ball_velocity, mut ball_position, ball_shape)) = ball.get_single_mut() {
        let bp = ball_position.0;
        if bp.y > GAME_HEIGHT / 2.0 {
            ball_velocity.0.y *= -1.0;
            ball_position.0.y = GAME_HEIGHT / 2.0;
        }
        if bp.y < -GAME_HEIGHT / 2.0 {
            ball_velocity.0.y *= -1.0;
            ball_position.0.y = -GAME_HEIGHT / 2.0;
        }

        for (paddle_position, paddle_shape) in paddles.iter() {
            let pp = paddle_position.0;
            let ps = paddle_shape.0;
            if pp.x < 0.0
                && bp.x - BALL_SIZE < pp.x + ps.x / 2.0
                && bp.x + BALL_SIZE > pp.x - ps.x
                && bp.y < pp.y + ps.y / 2.0
                && bp.y > pp.y - ps.y / 2.0
            {
                ball_velocity.0.x *= -1.0;
                ball_position.0.x = pp.x + ps.x / 2.0 + BALL_SIZE;
            }
            if pp.x > 0.0
                && bp.x + BALL_SIZE > pp.x - ps.x / 2.0
                && bp.x - BALL_SIZE < pp.x + ps.x
                && bp.y < pp.y + ps.y / 2.0
                && bp.y > pp.y - ps.y / 2.0
            {
                ball_velocity.0.x *= -1.0;
                ball_position.0.x = pp.x - ps.x / 2.0 - BALL_SIZE;
            }
        }
        ball_position.0 += ball_velocity.0;
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
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 2.0), // z index?
                    ..default()
                },
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
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.0), // z index?
                ..default()
            },
            ..default()
        },
    ));
}
