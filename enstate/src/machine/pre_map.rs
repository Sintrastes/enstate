//! Pre-map combinator for input transformation.
//!
//! This module provides the `PreMap` state machine, which transforms
//! input transitions before passing them to an inner machine. This is
//! useful for adapting machine interfaces and composing machines with
//! different transition types.
//!

use crate::machine::Machine;
use core::marker::PhantomData;

/// A state machine that transforms input before passing to inner machine.
///
/// This machine wraps another machine and applies a transformation function
/// to all incoming transitions before forwarding them to the inner machine.
/// The state type remains the same as the inner machine.
///
/// # Type Parameters
/// - `M` - The inner machine type
/// - `F` - The transformation function type
/// - `I` - The input transition type (before transformation)
/// - `T` - The state type (from inner machine)
#[derive(Debug, Clone, Copy)]
pub(super) struct PreMap<M, F, I, T> {
    machine: M,
    f: F,
    _phantom_i: PhantomData<I>,
    _phantom_t: PhantomData<T>,
}

impl<M, F, I, T> PreMap<M, F, I, T> {
    /// Creates a new pre-map machine.
    ///
    /// # Arguments
    /// - `machine` - The inner machine to wrap
    /// - `f` - The transformation function for input transitions
    pub(super) fn new(machine: M, f: F) -> Self {
        Self {
            machine,
            f,
            _phantom_i: PhantomData,
            _phantom_t: PhantomData,
        }
    }
}

impl<M, F, X: Send, I: Send + Sync + Clone, T: Send + Sync + Clone> Machine<I, T>
    for PreMap<M, F, I, T>
where
    M: Machine<X, T>,
    F: FnMut(&I) -> X + Send + Sync + Clone,
{
    fn edges(&self) -> impl Iterator<Item = I> {
        // PreMap machines accept any input of type I
        core::iter::empty()
    }

    fn state(&mut self) -> &T {
        self.machine.state()
    }

    fn traverse(&mut self, event: I) {
        let transformed = (self.f)(&event);
        self.machine.traverse(transformed);
    }

    fn traverse_ref(&mut self, event: &I) {
        let transformed = (self.f)(event);
        self.machine.traverse(transformed);
    }
}
