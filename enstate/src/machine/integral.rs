//! Integration combinator for accumulating values over time.
//!
//! This module provides the `Integral` state machine, which integrates
//! its input over time using the trapezoid rule. This is analogous to
//! Yampa's `integral` combinator.
//!
//! # Example
//! ```
//! use enstate::machine::Machine;
//! use enstate::machine::integral::{integral};
//! use enstate::integration::{Vec2, DeltaTime};
//!
//! // Integrate velocity to get position
//! let mut position = integral(Vec2::zero());
//!
//! position.traverse((Vec2::new(1.0, 0.0), DeltaTime::from_seconds(0.1)));
//! ```

use crate::integration::{DeltaTime, Integrable, IntegratedState};
use crate::machine::Machine;

/// A state machine that integrates its input over time using the trapezoid rule.
///
/// This combinator maintains an internal state that accumulates the integral
/// of incoming values over time. It uses the trapezoid rule for numerical
/// integration, which provides better accuracy than simple Euler integration.
///
/// # Type Parameters
/// - `T` - The type being integrated, must implement `Integrable`
#[derive(Debug, Clone, Copy)]
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

impl<T: Integrable + Send + Sync> Machine<(T, DeltaTime), T> for Integral<T> {
    fn edges(&self) -> impl Iterator<Item = (T, DeltaTime)> {
        // Integration machines accept any input
        core::iter::empty()
    }

    fn state(&mut self) -> &T {
        &self.state.value
    }

    fn traverse(&mut self, event: (T, DeltaTime)) {
        let (value, dt) = event;
        self.state.update(value, dt);
    }

    fn traverse_ref(&mut self, event: &(T, DeltaTime)) {
        let (value, dt) = event;
        self.state.update(*value, *dt);
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
pub fn integral<T: Integrable + Send + Sync>(
    initial: T,
) -> impl Machine<(T, DeltaTime), T> + Clone {
    Integral::new(initial)
}
