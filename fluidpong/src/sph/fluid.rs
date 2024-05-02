use std::iter::Sum;
use std::ops::Mul;

use bevy::ecs::component::Component;
use bevy::math::{Vec2, Vec4};
use rayon::prelude::*;

use crate::sph::kernel::{Kernel, KernelFunction, Poly6Kernel, SpikyKernel, ViscosityKernel};
use crate::sph::particle::Particle;
use crate::sph::spatial_grid::SpatialGrid2D;
use crate::{GAME_HEIGHT, GAME_WIDTH};

const NUM_PARTICLES_X: u32 = 36;
const NUM_PARTICLES_Y: u32 = 48;

pub const WALL_X: f32 = GAME_WIDTH / 2.0;
pub const WALL_Y: f32 = GAME_HEIGHT / 2.0;

const EPS: f32 = 1.0;
pub const NUM_PARTICLES: usize = (NUM_PARTICLES_X * NUM_PARTICLES_Y) as usize;

#[derive(Component)]
pub struct Fluid {
    particles: SpatialGrid2D<Particle>,
    density_kernel: Kernel,
    pressure_kernel: Kernel,
    viscosity_kernel: Kernel,
}

impl Fluid {
    /// Creates a new fluid simulation with a grid of particles.
    pub fn new(kernel_radius: f32, particle_mass: f32) -> Self {
        let mut particles = SpatialGrid2D::new(kernel_radius);

        let dx = WALL_X * 2.0 / NUM_PARTICLES_X as f32;
        let dy = WALL_Y * 2.0 / NUM_PARTICLES_Y as f32;
        for i in 0..NUM_PARTICLES_X {
            for j in 0..NUM_PARTICLES_Y {
                let x = -WALL_X * 0.9 + i as f32 * dx * 0.9;
                let y = -WALL_Y * 0.9 + j as f32 * dy * 0.9;
                particles.insert(Particle::new(Vec2::new(x, y), particle_mass));
            }
        }
        Self {
            particles,
            density_kernel: Poly6Kernel::new(kernel_radius).into(),
            pressure_kernel: SpikyKernel::new(kernel_radius).into(),
            viscosity_kernel: ViscosityKernel::new(kernel_radius).into(),
        }
    }

    pub fn reset(&mut self, kernel_radius: f32, particle_mass: f32) {
        let mut particles = SpatialGrid2D::new(kernel_radius);
        let dx = WALL_X * 2.0 / NUM_PARTICLES_X as f32;
        let dy = WALL_Y * 2.0 / NUM_PARTICLES_Y as f32;
        for i in 0..NUM_PARTICLES_X {
            for j in 0..NUM_PARTICLES_Y {
                let x = -WALL_X * 0.9 + i as f32 * dx * 0.9;
                let y = -WALL_Y * 0.9 + j as f32 * dy * 0.9;
                particles.insert(Particle::new(Vec2::new(x, y), particle_mass));
            }
        }
        self.particles = particles;
    }

    /// Interpolates a value at the given position. The value is extracted using
    /// the given function `f` on each particle, and then weighted using the
    /// given kernel function.
    fn interpolate<T>(&self, pos: Vec2, kernel: &Kernel, f: impl Fn(&Particle) -> T) -> T
    where
        T: Mul<f32, Output = T> + Sum,
    {
        (self.particles.query(pos).into_iter())
            .map(|p| f(p) * (1.0 / p.density) * kernel.evaluate(pos - p.position))
            .sum()
    }

    /// Interpolates a gradient at the given position. The value is extracted
    /// using the given function `f` on each particle, and then weighted using
    /// the given kernel function.
    fn interpolate_grad(&self, pos: Vec2, kernel: &Kernel, f: impl Fn(&Particle) -> f32) -> Vec2 {
        (self.particles.query(pos).into_iter())
            .filter(|p| p.position != pos)
            .map(|p| f(p) * (1.0 / p.density) * kernel.gradient(pos - p.position))
            .sum()
    }

    /// Interpolates a Laplacian at the given position. The value is extracted
    /// using the given function `f` on each particle, and then weighted using
    /// the given kernel function.
    fn interpolate_lapl<T>(&self, pos: Vec2, kernel: &Kernel, f: impl Fn(&Particle) -> T) -> T
    where
        T: Mul<f32, Output = T> + Sum,
    {
        (self.particles.query(pos).into_iter())
            .filter(|p| p.position != pos)
            .map(|p| f(p) * (1.0 / p.density) * kernel.laplacian(pos - p.position))
            .sum()
    }

    /// Computes the density and pressure of each particle based on the current
    /// state of the simulation.
    pub fn compute_density_pressure(&mut self, gas_const: f32, rest_dens: f32) {
        // First, set densities to 1 so we can cancel it in the interpolation
        self.particles.iter_mut().par_bridge().for_each(|pi| {
            pi.density = 1.0;
        });

        // FIXME: Can we avoid this clone?
        let mut new_particles = self.particles.clone();
        new_particles.iter_mut().par_bridge().for_each(|pi| {
            pi.density = self.interpolate(pi.position, &self.density_kernel, |pj| pj.mass);
            pi.pressure = gas_const * (pi.density - rest_dens);
        });

        self.particles = new_particles;
    }

    /// Computes the forces acting on each particle based on the current state
    /// of the simulation.
    pub fn compute_forces(&mut self, visc_const: f32, gravity: f32) {
        // FIXME: Can we avoid this clone?
        let mut new_particles = self.particles.clone();

        new_particles.iter_mut().par_bridge().for_each(|pi| {
            // Define the extraction functions for interpolating pressure and viscosity
            let ext_press = |pj: &Particle| -pj.mass * (pi.pressure + pj.pressure) / 2.0;
            let ext_visc = |pj: &Particle| visc_const * pj.mass * (pj.velocity - pi.velocity);

            // Compute the forces acting on the particle
            let fi_press = self.interpolate_grad(pi.position, &self.pressure_kernel, ext_press);
            let fi_visc = self.interpolate_lapl(pi.position, &self.viscosity_kernel, ext_visc);
            let fi_gravity = Vec2::new(0.0, -gravity) * pi.mass;
            pi.force = fi_press + fi_visc + fi_gravity + pi.ext_force;
        });

        self.particles = new_particles;
    }

    /// Updates the fluid simulation based on current forces by one time step.
    pub fn integrate(&mut self, dt: f32, bound_damping: f32) {
        let mut new_particles = self.particles.clone();

        new_particles.iter_mut().par_bridge().for_each(|pi| {
            // Euler
            pi.velocity += dt * pi.force / pi.density;
            pi.position += dt * pi.velocity;

            if pi.position.x - EPS < -WALL_X {
                pi.velocity.x *= -bound_damping;
                pi.position.x = EPS - WALL_X;
            }
            if pi.position.x + EPS > WALL_X {
                pi.velocity.x *= -bound_damping;
                pi.position.x = WALL_X - EPS;
            }
            if pi.position.y - EPS < -WALL_Y {
                pi.velocity.y *= -bound_damping;
                pi.position.y = EPS - WALL_Y;
            }
            if pi.position.y + EPS > WALL_Y {
                pi.velocity.y *= -bound_damping;
                pi.position.y = WALL_Y - EPS;
            }
        });

        new_particles.recompute();
        self.particles = new_particles;
    }

    /// Sets the external force acting on the fluid at the given point.
    pub fn set_external_force(&mut self, point: Vec2, force: Vec2, radius: f32) {
        for particle in self.particles.iter_mut() {
            let distance = (particle.position.distance(point) - radius).max(0.0);
            let logistic_response = 1.0 / (1.0 + f32::exp(1.0 + distance));
            particle.ext_force = force * logistic_response;
            // particle.ext_force = force * self.density_kernel.evaluate(particle.position - point);
        }
    }

    /// Add the external force acting on the fluid at the given point.
    pub fn add_external_force(&mut self, point: Vec2, force: Vec2, radius: f32) {
        for particle in self.particles.iter_mut() {
            let distance = (particle.position.distance(point) - radius).max(0.0);
            let logistic_response = 1.0 / (1.0 + f32::exp(1.0 + distance));
            particle.ext_force += force * logistic_response;
            // particle.ext_force += force * self.density_kernel.evaluate(particle.position - point);
        }
    }

    pub fn get_force_at(&self, point: Vec2, velocity: Vec2) -> Vec2 {
        // let avg_vel = |pj: &Particle| pj.velocity;
        // let ext_visc = |pj: &Particle| 200.0 * pj.mass * (pj.velocity - velocity);
        // let fi = self.interpolate(point, &self.pressure_kernel, avg_vel);
        // return fi
        let mut f: Vec2 = Vec2::ZERO;
        for particle in self.particles.iter() {
            let distance = (particle.position.distance(point) - 3.0).max(0.0);
            let logistic_response = 1.0 / (1.0 + f32::exp(1.0 + distance));
            f += particle.velocity * logistic_response;
        }
        return f - velocity;
    }

    /// Returns a reference to the particles in the fluid.
    pub fn particles(&self) -> &SpatialGrid2D<Particle> {
        &self.particles
    }

    /// Returns metaball information for the shader
    pub fn get_balls(&self) -> [Vec4; NUM_PARTICLES] {
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
