use bevy::{ecs::component::Component, math::Vec4};

use super::math::index;

pub const INTERACT_VELOCITY: f32 = 5000.0;
pub const FLUID_SIZE: u32 = 64;
pub const NUM_CELLS: usize = (FLUID_SIZE * FLUID_SIZE) as usize;
pub const WIDTH: f32 = 300.0;
pub const HEIGHT: f32 = 300.0;

#[derive(Component)]
pub struct Fluid {
    pub size: u32,
    pub s: Vec<f32>,
    pub density: Vec<f32>,
    pub vx: Vec<f32>,
    pub vy: Vec<f32>,
    pub vx0: Vec<f32>,
    pub vy0: Vec<f32>,
}

impl Fluid {
    pub fn new(size: u32) -> Fluid {
        let num_cells = (size * size) as usize;
        Fluid {
            size,
            s: vec![0.0; num_cells],
            density: vec![0.0; num_cells],
            vx: vec![0.0; num_cells],
            vy: vec![0.0; num_cells],
            vx0: vec![0.0; num_cells],
            vy0: vec![0.0; num_cells],
        }
    }

    pub fn reset(&mut self) {
        let num_cells = (self.size * self.size) as usize;
        self.s = vec![0.0; num_cells];
        self.density = vec![0.0; num_cells];
        self.vx = vec![0.0; num_cells];
        self.vy = vec![0.0; num_cells];
        self.vx0 = vec![0.0; num_cells];
        self.vy0 = vec![0.0; num_cells];
    }

    pub fn add_density(&mut self, x: u32, y: u32, amount: f32) {
        self.density[index(self.size, x, y)] += amount;
    }

    pub fn add_velocity(&mut self, x: u32, y: u32, amount_x: f32, amount_y: f32) {
        let index = index(self.size, x, y);

        self.vx[index] += amount_x;
        self.vy[index] += amount_y;
    }

    pub fn get_cells(&self) -> [Vec4; NUM_CELLS] {
        let mut cells = [Vec4::ZERO; NUM_CELLS];
        for i in 0..NUM_CELLS {
            cells[i] = Vec4::new(self.vx[i], self.vy[i], self.density[i], 0.0);
        }
        cells
    }
}