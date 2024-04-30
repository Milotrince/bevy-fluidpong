use bevy::prelude::*;

pub trait PongFluid {
    fn apply_emit_force(&mut self, position: Vec2, velocity: Vec2);
    fn apply_paddle_force(&mut self, position: Vec2, velocity: Vec2);
    fn apply_ball_force(&mut self, position: Vec2, velocity: Vec2);
    fn get_fluid_force_at(&self, position: Vec2, velocity: Vec2) -> Vec2;
}