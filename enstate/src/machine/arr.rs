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
//! let mut velocity = arr(|angle: &f32| {
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
#[derive(Debug, Clone, Copy)]
pub struct Arr<I: Clone, O: Clone, F: Send> {
    f: F,
    last_output: Option<O>,
    _phantom: PhantomData<I>,
}

impl<I: Clone + Send + Sync, O: Clone + Send + Sync, F: FnMut(&I) -> O + Send + Sync + Clone>
    Machine<I, O> for Arr<I, O, F>
{
    fn edges(&self) -> impl Iterator<Item = I> {
        // Arrow machines accept any input
        core::iter::empty()
    }

    fn state(&mut self) -> &O {
        &self.last_output.as_ref().unwrap()
    }

    fn traverse(&mut self, event: I) {
        self.last_output = Some((self.f)(&event));
    }

    fn traverse_ref(&mut self, event: &I) {
        self.last_output = Some((self.f)(event));
    }
}

/// Creates a stateless transformation machine.
///
/// # Arguments
/// - `f` - The transformation function to apply to each input
///
pub fn arr<
    I: Clone + Send + Sync,
    O: Clone + Send + Sync,
    F: FnMut(&I) -> O + Send + Sync + Clone,
>(
    f: F,
) -> impl Machine<I, O> + Clone {
    Arr {
        f,
        last_output: None,
        _phantom: PhantomData,
    }
}
