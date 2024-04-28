use std::f32::consts::PI;

use bevy::math::Vec2;
use enum_dispatch::enum_dispatch;

/// A kernel function for SPH fluid simulations.
///
/// Kernel functions specify the weight of a particle's influence on a point in
/// space. They are used to calculate continuous density and pressure based on
/// discrete particle properties.
#[enum_dispatch]
pub trait KernelFunction {
    /// Evaluates the kernel function at the given displacement.
    fn evaluate(&self, _: Vec2) -> f32 {
        unimplemented!()
    }

    /// Evaluates the gradient of the kernel function at the given displacement.
    fn gradient(&self, _: Vec2) -> Vec2 {
        unimplemented!()
    }

    /// Evaluates the Laplacian of the kernel function at the given displacement.
    fn laplacian(&self, _: Vec2) -> f32 {
        unimplemented!()
    }
}

/// An enum that can hold one of many types that implement the `KernelFunction`
/// trait. This is useful for storing different types of kernel functions in a
/// single data structure.
#[enum_dispatch(KernelFunction)]
pub enum Kernel {
    Poly6(Poly6Kernel),
    Spiky(SpikyKernel),
    Viscosity(ViscosityKernel),
}

/// A good general-purpose kernel for SPH fluid simulations that avoids
/// instability when particles are too close together.
#[derive(Debug)]
pub struct Poly6Kernel {
    h: f32,
}

impl Poly6Kernel {
    pub fn new(h: f32) -> Self {
        Self { h }
    }

    pub fn coefficient(&self, r: Vec2) -> f32 {
        if r.length() <= self.h {
            // 315.0 / (64.0 * PI * self.h.powi(9))
            4.0 / (PI * self.h.powi(8))
        } else {
            0.0
        }
    }
}

impl KernelFunction for Poly6Kernel {
    fn evaluate(&self, r: Vec2) -> f32 {
        let h2 = self.h.powi(2);
        let r2 = r.length_squared();

        self.coefficient(r) * (h2 - r2).powi(3)
    }

    fn gradient(&self, r: Vec2) -> Vec2 {
        let h2 = self.h.powi(2);
        let r2 = r.length_squared();

        self.coefficient(r) * -6.0 * (h2 - r2).powi(2) * r
    }

    fn laplacian(&self, r: Vec2) -> f32 {
        let h2 = self.h.powi(2);
        let h4 = h2.powi(4);
        let r2 = r.length_squared();
        let r4 = r2.powi(2);

        self.coefficient(r) * -6.0 * (3.0 * h4 - 10.0 * h2 * r2 + 7.0 * r4)
    }
}

/// A kernel function that peaks near 0, good for pressure calculations.
#[derive(Debug)]
pub struct SpikyKernel {
    h: f32,
}

impl SpikyKernel {
    pub fn new(h: f32) -> Self {
        Self { h }
    }
}

impl KernelFunction for SpikyKernel {
    fn gradient(&self, r: Vec2) -> Vec2 {
        if r.length() <= self.h {
            let h5 = self.h.powi(5);
            -10.0 / (PI * h5) * (self.h - r.length()).powi(2) * r.normalize()
        } else {
            Vec2::ZERO
        }
    }
}

/// A kernel function that is used to calculate viscosity.
#[derive(Debug)]
pub struct ViscosityKernel {
    h: f32,
}

impl ViscosityKernel {
    pub fn new(h: f32) -> Self {
        Self { h }
    }
}

impl KernelFunction for ViscosityKernel {
    fn laplacian(&self, r: Vec2) -> f32 {
        if r.length() <= self.h {
            let h5 = self.h.powi(5);
            40.0 / (PI * h5) * (self.h - r.length())
        } else {
            0.0
        }
    }
}
