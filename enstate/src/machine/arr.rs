//! Stateless transformation combinator.
//!
//! This module provides the `Arr` state machine, which applies a stateless
//! function to its input. This is analogous to Yampa's `arr` combinator.
//!
//! # Example
//! ```
//! use enstate::machine::Machine;
//! use enstate::machine::arr::arr;
//! use enstate::integration::Vec2;
//!
//! // Convert direction angle to velocity vector
//! let mut velocity = arr(Vec2::zero(), |angle: &f32| {
//!     Vec2::new(angle.cos(), angle.sin())
//! });
//!
//! velocity.traverse(0.0);
//! assert_eq!(*velocity.state(), Vec2::new(1.0, 0.0));
//! ```

use crate::machine::Machine;
use core::marker::PhantomData;

/// A state machine that applies a stateless function to its input.
///
/// This machine transforms each input event through a function and stores
/// the result as its state. It has no internal state other than the last
/// computed output.
///
/// # Type Parameters
/// - `I` - The input type
/// - `O` - The output type
/// - `F` - The transformation function type
pub struct Arr<I: Clone, O: Clone, F> {
    f: F,
    last_output: O,
    _phantom: PhantomData<I>,
}

impl<I: Clone, O: Clone, F: FnMut(&I) -> O> Machine<O> for Arr<I, O, F> {
    type Transition = I;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        // Arrow machines accept any input
        core::iter::empty()
    }

    fn state(&self) -> &O {
        &self.last_output
    }

    fn traverse(&mut self, event: Self::Transition) {
        self.last_output = (self.f)(&event);
    }

    fn traverse_ref(&mut self, event: &Self::Transition) {
        self.last_output = (self.f)(event);
    }
}

/// Creates a stateless transformation machine.
///
/// # Arguments
/// - `initial` - The initial output value (before any input is processed)
/// - `f` - The transformation function to apply to each input
///
pub fn arr<I: Clone, O: Clone, F: FnMut(&I) -> O>(initial: O, f: F) -> Arr<I, O, F> {
    Arr {
        f,
        last_output: initial,
        _phantom: PhantomData,
    }
}
