use bevy::prelude::*;

use crate::pong::pongfluid::PongFluid;

pub const BALL_FORCE_ON_FLUID: f32 = 1000.0;
pub const BALL_FORCE_ON_FLUID_RADIUS: f32 = 5.0;
pub const PADDLE_FORCE_ON_FLUID: f32 = 10000.0;
pub const PADDLE_FORCE_ON_FLUID_RADIUS: f32 = 10.0;
pub const EMIT_FORCE_ON_FLUID: f32 = 100000.0;
pub const EMIT_FORCE_ON_FLUID_RADIUS: f32 = 40.0;
pub const FLUID_FORCE_ON_BALL: f32 = 0.00001;

impl PongFluid for crate::sph::fluid::Fluid {
     fn apply_emit_force(&mut self, position: Vec2, velocity: Vec2) {
        self.add_external_force(position, velocity * EMIT_FORCE_ON_FLUID, EMIT_FORCE_ON_FLUID_RADIUS);
     }
    fn apply_paddle_force(&mut self, position: Vec2, velocity: Vec2) {
        self.add_external_force(position, velocity * PADDLE_FORCE_ON_FLUID, PADDLE_FORCE_ON_FLUID_RADIUS);
    }
    fn apply_ball_force(&mut self, position: Vec2, velocity: Vec2) {
        self.set_external_force(position, velocity * BALL_FORCE_ON_FLUID, BALL_FORCE_ON_FLUID_RADIUS);
    }
    fn get_fluid_force_at(&self, position: Vec2, velocity: Vec2) -> Vec2 {
        return self.get_force_at(position, velocity) * FLUID_FORCE_ON_BALL;
    }
}
