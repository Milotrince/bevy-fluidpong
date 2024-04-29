use std::iter::Sum;
use std::ops::Mul;

use bevy::ecs::component::Component;
use bevy::math::Vec2;

use crate::sph::kernel::{Kernel, KernelFunction, Poly6Kernel, SpikyKernel, ViscosityKernel};
use crate::sph::particle::Particle;
use crate::sph::spatial_grid::SpatialGrid2D;

const KERNEL_RADIUS: f32 = 8.0;
const NUM_PARTICLES_X: u32 = 32;
const NUM_PARTICLES_Y: u32 = 32;
const PARTICLE_MASS: f32 = 100.0;
const REST_DENS: f32 = 5.0;
const GAS_CONST: f32 = 10000.0;
const VISC_CONST: f32 = 200.0;
const EPS: f32 = KERNEL_RADIUS;
const BOUND_DAMPING: f32 = -0.5;

#[derive(Component)]
pub struct Fluid {
    particles: SpatialGrid2D<Particle>,
    density_kernel: Kernel,
    pressure_kernel: Kernel,
    viscosity_kernel: Kernel,
}

impl Fluid {
    /// Creates a new fluid simulation with a grid of particles.
    pub fn new() -> Self {
        let mut particles = SpatialGrid2D::new(KERNEL_RADIUS);

        for i in 0..NUM_PARTICLES_X {
            for j in 0..NUM_PARTICLES_Y {
                let x = -100.0 + i as f32 * 4.0;
                let y = -64.0 + j as f32 * 4.0;
                // if x * x + y * y > 80.0 * 80.0 {
                //     continue;
                // }
                particles.insert(Particle::new(Vec2::new(x, y), PARTICLE_MASS));
            }
        }

        Self {
            particles,
            density_kernel: Poly6Kernel::new(KERNEL_RADIUS).into(),
            pressure_kernel: SpikyKernel::new(KERNEL_RADIUS).into(),
            viscosity_kernel: ViscosityKernel::new(KERNEL_RADIUS).into(),
        }
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
    pub fn compute_density_pressure(&mut self) {
        // First, set densities to 1 so we can cancel it in the interpolation
        for pi in self.particles.iter_mut() {
            pi.density = 1.0;
        }

        // FIXME: Can we avoid this clone?
        let mut new_particles = self.particles.clone();
        for pi in new_particles.iter_mut() {
            pi.density = self.interpolate(pi.position, &self.density_kernel, |pj| pj.mass);
            pi.pressure = GAS_CONST * (pi.density - REST_DENS);
        }

        self.particles = new_particles;
    }

    /// Computes the forces acting on each particle based on the current state
    /// of the simulation.
    pub fn compute_forces(&mut self) {
        // FIXME: Can we avoid this clone?
        let mut new_particles = self.particles.clone();

        for pi in new_particles.iter_mut() {
            // Define the extraction functions for interpolating pressure and viscosity
            let ext_press = |pj: &Particle| -pj.mass * (pi.pressure + pj.pressure) / 2.0;
            let ext_visc = |pj: &Particle| VISC_CONST * pj.mass * (pj.velocity - pi.velocity);

            // Compute the forces acting on the particle
            let fi_press = self.interpolate_grad(pi.position, &self.pressure_kernel, ext_press);
            let fi_visc = self.interpolate_lapl(pi.position, &self.viscosity_kernel, ext_visc);
            let fi_gravity = Vec2::new(0.0, -10.0) * pi.mass;
            pi.force = fi_press + fi_visc + fi_gravity + pi.ext_force;
        }

        self.particles = new_particles;
    }

    /// Updates the fluid simulation based on current forces by one time step.
    pub fn integrate(&mut self, dt: f32) {
        let mut new_particles = self.particles.clone();

        for pi in new_particles.iter_mut() {
            // Euler
            pi.velocity += dt * pi.force / pi.density;
            pi.position += dt * pi.velocity;

            if pi.position.x - EPS < -100.0 {
                pi.velocity.x *= BOUND_DAMPING;
                pi.position.x = EPS - 100.0;
            }
            if pi.position.x + EPS > 100.0 {
                pi.velocity.x *= BOUND_DAMPING;
                pi.position.x = 100.0 - EPS;
            }
            if pi.position.y - EPS < -200.0 {
                pi.velocity.y *= BOUND_DAMPING;
                pi.position.y = EPS - 200.0;
            }
            if pi.position.y + EPS > 200.0 {
                pi.velocity.y *= BOUND_DAMPING;
                pi.position.y = 200.0 - EPS;
            }
        }

        new_particles.recompute();
        self.particles = new_particles;
    }

    /// Sets the external force acting on the fluid at the given point.
    pub fn set_external_force(&mut self, point: Vec2, force: Vec2) {
        for particle in self.particles.iter_mut() {
            let distance = (particle.position.distance(point) - 10.0).max(0.0);
            let logistic_response = 1.0 / (1.0 + f32::exp(1.0 + distance));
            particle.ext_force = force * logistic_response;
        }
    }

    /// Returns a reference to the particles in the fluid.
    pub fn particles(&self) -> &SpatialGrid2D<Particle> {
        &self.particles
    }
}
