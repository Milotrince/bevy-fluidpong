use std::sync::Arc;

use bevy::math::Vec2;

use super::spatial_grid::Position;

/// A particle in the SPH simulation.
///
/// Each particle has a position, velocity, and mass. A particle does not
/// represent a physical object, like a molecule or water droplet, but rather a
/// sample of the continuous fluid at a specific point in space.
#[derive(Debug, Clone)]
pub struct Particle {
    pub mass: f32,
    pub position: Vec2,
    pub velocity: Vec2,
    pub ext_force: Vec2,

    pub density: f32,
    pub pressure: f32,
    pub force: Vec2,
}

impl Particle {
    /// Creates a new particle with the given position and mass.
    pub fn new(position: Vec2, mass: f32) -> Self {
        Self {
            mass,
            position,
            velocity: Vec2::ZERO,
            ext_force: Vec2::ZERO,
            density: 0.0,
            pressure: 0.0,
            force: Vec2::ZERO,
        }
    }
}

impl Position for Particle {
    fn position(&self) -> Vec2 {
        self.position
    }
}

impl Position for Arc<Particle> {
    fn position(&self) -> Vec2 {
        self.position
    }
}
