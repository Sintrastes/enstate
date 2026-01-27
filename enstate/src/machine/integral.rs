//! Integration combinator for accumulating values over time.
//!
//! This module provides the `Integral` state machine, which integrates
//! its input over time using the trapezoid rule. This is analogous to
//! Yampa's `integral` combinator.
//!
//! # Example
//! ```
//! use enstate::machine::Machine;
//! use enstate::machine::integral::{integral, IntegralInput};
//! use enstate::integration::Vec2;
//!
//! // Integrate velocity to get position
//! let mut position = integral(Vec2::zero());
//!
//! position.traverse(IntegralInput {
//!     value: Vec2::new(1.0, 0.0),
//!     dt: 0.1,
//! });
//! ```

use crate::integration::{Integrable, IntegratedState};
use crate::machine::Machine;

/// A state machine that integrates its input over time using the trapezoid rule.
///
/// This combinator maintains an internal state that accumulates the integral
/// of incoming values over time. It uses the trapezoid rule for numerical
/// integration, which provides better accuracy than simple Euler integration.
///
/// # Type Parameters
/// - `T` - The type being integrated, must implement `Integrable`
pub struct Integral<T: Integrable> {
    state: IntegratedState<T>,
}

impl<T: Integrable> Integral<T> {
    /// Creates a new integral machine with an initial value.
    ///
    /// # Arguments
    /// - `initial` - The starting value for the integration
    pub fn new(initial: T) -> Self
    where
        T: Copy,
    {
        Self {
            state: IntegratedState::new(initial, initial),
        }
    }
}

/// Input event for integration containing the value to integrate and time step.
#[derive(Debug, Clone, Copy)]
pub struct IntegralInput<T> {
    /// The value to integrate at this time step
    pub value: T,
    /// The time delta for this integration step
    pub dt: f32,
}

impl<T: Integrable> Machine<T> for Integral<T> {
    // TODO: Need to make this generic somehow?
    type Transition = IntegralInput<T>;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        // Integration machines accept any input
        core::iter::empty()
    }

    fn state(&self) -> &T {
        &self.state.value
    }

    fn traverse(&mut self, event: Self::Transition) {
        self.state.update(event.value, event.dt);
    }

    fn traverse_ref(&mut self, event: &Self::Transition) {
        self.state.update(event.value, event.dt);
    }
}

/// Creates an integrating state machine with an initial value.
///
/// This is a convenience function for constructing an `Integral` machine.
///
/// # Example
/// ```
/// use enstate::integration::Vec2;
/// use enstate::machine::integral::integral;
///
/// // Integrate acceleration to get velocity
/// let velocity = integral(Vec2::zero());
/// ```
pub fn integral<T: Integrable>(initial: T) -> Integral<T> {
    Integral::new(initial)
}
