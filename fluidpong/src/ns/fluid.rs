use bevy::{
    ecs::component::Component,
    math::{Vec2, Vec4},
};

use crate::{GAME_HEIGHT, GAME_WIDTH};

use super::math::index;

pub const INTERACT_VELOCITY: f32 = 5000.0;
pub const GRID_X: u32 = 96; //128;
pub const GRID_Y: u32 = 72;
pub const NUM_CELLS: usize = (GRID_X * GRID_Y) as usize;
pub const WIDTH: f32 = GAME_WIDTH;
pub const HEIGHT: f32 = GAME_HEIGHT;

#[derive(Component)]
pub struct Fluid {
    pub s: Vec<f32>,
    pub density: Vec<f32>,
    pub vx: Vec<f32>,
    pub vy: Vec<f32>,
    pub vx0: Vec<f32>,
    pub vy0: Vec<f32>,
}

impl Fluid {
    pub fn new() -> Fluid {
        Fluid {
            s: vec![0.0; NUM_CELLS],
            density: vec![0.0; NUM_CELLS],
            vx: vec![0.0; NUM_CELLS],
            vy: vec![0.0; NUM_CELLS],
            vx0: vec![0.0; NUM_CELLS],
            vy0: vec![0.0; NUM_CELLS],
        }
    }

    pub fn reset(&mut self) {
        self.s = vec![0.0; NUM_CELLS];
        self.density = vec![0.0; NUM_CELLS];
        self.vx = vec![0.0; NUM_CELLS];
        self.vy = vec![0.0; NUM_CELLS];
        self.vx0 = vec![0.0; NUM_CELLS];
        self.vy0 = vec![0.0; NUM_CELLS];
    }

    pub fn add_density(&mut self, position: Vec2, amount: f32) {
        let (i, j) = screen_to_grid(position);
        self.add_density_grid(i, j, amount)
    }

    pub fn add_density_grid(&mut self, i: u32, j: u32, amount: f32) {
        self.density[index(i, j)] += amount;
    }

    pub fn add_velocity(&mut self, position: Vec2, amount: Vec2) {
        let (i, j) = screen_to_grid(position);
        self.add_velocity_grid(i, j, amount.x, amount.y)
    }

    pub fn add_velocity_grid(&mut self, x: u32, y: u32, amount_x: f32, amount_y: f32) {
        let i = index(x, y);
        let d = self.density[i];
        self.vx[i] += amount_x * d;
        self.vy[i] += amount_y * d;
    }

    pub fn get_density_at(&self, position: Vec2) -> f32 {
        let (i, j) = screen_to_grid(position);
        return self.density[index(i, j)];
    }

    pub fn get_velocity_at(&self, position: Vec2) -> Vec2 {
        let (x, y) = screen_to_grid(position);
        let i = index(x, y);
        return Vec2::new(self.vx[i], self.vy[i]);
    }

    pub fn get_cells(&self) -> [Vec4; NUM_CELLS] {
        let mut cells = [Vec4::ZERO; NUM_CELLS];
        for i in 0..NUM_CELLS {
            cells[i] = Vec4::new(self.vx[i], self.vy[i], self.density[i], 0.0);
        }
        cells
    }
}

fn screen_to_grid(position: Vec2) -> (u32, u32) {
    let i = ((position.x + WIDTH / 2.0) / (WIDTH as f32) * (GRID_X as f32)) as u32;
    let j = ((position.y + HEIGHT / 2.0) / (HEIGHT as f32) * (GRID_Y as f32)) as u32;
    return (i, j);
}
