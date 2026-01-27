//! Integration utilities for smooth continuous physics using the trapezoid rule.
//!
//! These utilities allow building state machines that integrate velocities
//! into positions smoothly over time, which is essential for game physics.

use core::ops::{Add, Mul};

/// Trait for types that support trapezoid rule integration.
pub trait Integrable: Copy + Add<Output = Self> + Mul<f32, Output = Self> {
    /// Integrate this value over time using the trapezoid rule.
    ///
    /// Given the previous derivative and current derivative, compute the
    /// integrated value over the time step `dt`.
    fn integrate_trapezoid(
        &self,
        prev_derivative: &Self,
        current_derivative: &Self,
        dt: f32,
    ) -> Self {
        // Trapezoid rule: x_new = x_old + dt * (v_old + v_new) / 2
        let avg_derivative = *prev_derivative + *current_derivative;
        *self + avg_derivative * (dt / 2.0)
    }
}

/// A 2D vector for position and velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn length(&self) -> f32 {
        libm::sqrtf(self.x * self.x + self.y * self.y)
    }

    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0 {
            Self {
                x: self.x / len,
                y: self.y / len,
            }
        } else {
            *self
        }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Integrable for Vec2 {}

impl Integrable for f32 {}

/// State that tracks both a value and its derivative for integration.
#[derive(Debug, Clone)]
pub struct IntegratedState<T: Integrable> {
    pub value: T,
    pub derivative: T,
    pub prev_derivative: T,
}

impl<T: Integrable> IntegratedState<T> {
    pub fn new(initial_value: T, initial_derivative: T) -> Self {
        Self {
            value: initial_value,
            derivative: initial_derivative,
            prev_derivative: initial_derivative,
        }
    }

    /// Update the derivative and integrate to get the new value.
    pub fn update(&mut self, new_derivative: T, dt: f32) {
        self.value = self
            .value
            .integrate_trapezoid(&self.prev_derivative, &new_derivative, dt);
        self.prev_derivative = self.derivative;
        self.derivative = new_derivative;
    }
}
