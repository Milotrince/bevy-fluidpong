use bevy::prelude::*;

use crate::ns;
use crate::pong::pongfluid::PongFluid;

const EMIT_DENSITY: f32 = 10.0;
const EMIT_VELOCITY: f32 = 10000.0;
const PADDLE_VELOCITY: f32 = 10.0;
const BALL_VELOCITY: f32 = 5.0;
const FLUID_ON_BALL_VELOCITY: f32 = 0.0001;
const FLUID_ON_BALL_DENSITY: f32 = 0.000001;

impl PongFluid for ns::fluid::Fluid {
     fn apply_emit_force(&mut self, position: Vec2, velocity: Vec2) {
        self.add_density(position, EMIT_DENSITY);
        self.add_velocity(position, velocity * EMIT_VELOCITY);
     }
    fn apply_paddle_force(&mut self, position: Vec2, velocity: Vec2) {
        self.add_velocity(position, velocity * PADDLE_VELOCITY);
    }
    fn apply_ball_force(&mut self, position: Vec2, velocity: Vec2) {
        self.add_velocity(position, velocity * BALL_VELOCITY);
    }
    fn get_fluid_force_at(&self, position: Vec2, velocity: Vec2) -> Vec2 {
        return self.get_velocity_at(position) * FLUID_ON_BALL_VELOCITY - self.get_density_at(position) * velocity * FLUID_ON_BALL_DENSITY;
    }
}
