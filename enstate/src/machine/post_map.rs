//! Post-map combinator for output transformation.
//!
//! This module provides the `PostMap` state machine, which transforms
//! the output state of an inner machine. This is useful for adapting
//! machine outputs and composing machines with different state types.
//!

use crate::machine::Machine;
use core::cell::RefCell;
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
pub(super) struct PostMap<M, F, T, U> {
    machine: M,
    f: F,
    cached_state: RefCell<Option<U>>,
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
            cached_state: RefCell::new(None),
            _phantom_t: PhantomData,
            _phantom_u: PhantomData,
        }
    }
}

impl<M, F, T, U> Machine<U> for PostMap<M, F, T, U>
where
    M: Machine<T>,
    F: Fn(&T) -> U,
{
    type Transition = M::Transition;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        self.machine.edges()
    }

    fn state(&self) -> &U {
        let inner_state = self.machine.state();
        let mut cache = self.cached_state.borrow_mut();
        let mapped = (self.f)(inner_state);
        *cache = Some(mapped);
        drop(cache);

        unsafe {
            // SAFETY: We just updated the cache above, so it contains Some(U)
            // The reference is valid for the lifetime of self
            let cache_ref = self.cached_state.as_ptr();
            (*cache_ref).as_ref().unwrap()
        }
    }

    fn traverse(&mut self, event: Self::Transition) {
        self.machine.traverse(event);
        // Clear cache to force recomputation on next state() call
        *self.cached_state.borrow_mut() = None;
    }

    fn traverse_ref(&mut self, event: &Self::Transition)
    where
        Self::Transition: Clone,
    {
        self.machine.traverse_ref(event);
        // Clear cache to force recomputation on next state() call
        *self.cached_state.borrow_mut() = None;
    }
}
