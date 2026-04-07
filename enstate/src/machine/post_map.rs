//! Post-map combinator for output transformation.
//!
//! This module provides the `PostMap` state machine, which transforms
//! the output state of an inner machine. This is useful for adapting
//! machine outputs and composing machines with different state types.
//!

use crate::machine::Machine;
use core::marker::PhantomData;

/// A state machine that transforms output after getting from inner machine.
///
/// This machine wraps another machine and applies a transformation function
/// to its state whenever it's accessed. The transformation is cached and
/// recomputed only when the inner machine's state changes.
///
/// # Type Parameters
/// - `M` - The inner machine type
/// - `F` - The transformation function type
/// - `T` - The inner machine's state type
/// - `U` - The output state type (after transformation)
#[derive(Debug, Clone, Copy)]
pub(super) struct PostMap<M, F, T, U> {
    machine: M,
    f: F,
    cached_state: Option<U>,
    _phantom_t: PhantomData<T>,
    _phantom_u: PhantomData<U>,
}

impl<M, F, T, U> PostMap<M, F, T, U> {
    /// Creates a new post-map machine.
    ///
    /// # Arguments
    /// - `machine` - The inner machine to wrap
    /// - `f` - The transformation function for the output state
    pub(super) fn new(machine: M, f: F) -> Self {
        Self {
            machine,
            f,
            cached_state: None,
            _phantom_t: PhantomData,
            _phantom_u: PhantomData,
        }
    }
}

impl<M, F, I: Send + Sync, T: Send + Sync + Clone, U: Send + Sync + Clone> Machine<I, U>
    for PostMap<M, F, T, U>
where
    M: Machine<I, T>,
    F: Fn(&T) -> U + Send + Sync + Clone,
{
    fn edges(&self) -> impl Iterator<Item = I> {
        self.machine.edges()
    }

    fn state(&mut self) -> &U {
        let inner_state = self.machine.state();
        let mapped = (self.f)(inner_state);
        self.cached_state = Some(mapped);

        &self.cached_state.as_ref().unwrap()
    }

    fn traverse(&mut self, event: I) {
        self.machine.traverse(event);
    }

    fn traverse_ref(&mut self, event: &I) {
        self.machine.traverse_ref(event);
    }
}
