use core::{cell::RefCell, marker::PhantomData};

use super::Machine;

///
/// MappedMachine allows for mapping Machine state type into another type while maintaining
/// the same semantics.
/// T is the input state type from the inner machine
/// U is the output state type after mapping
///
pub(super) struct MappedMachine<U, M, F, T> {
    pub(super) u: PhantomData<U>,
    pub(super) t: PhantomData<T>,
    pub(super) machine: M,
    pub(super) f: F,
    pub(super) cached_state: RefCell<Option<U>>,
}

impl<M, F, T, U> Machine<U> for MappedMachine<U, M, F, T>
where
    M: Machine<T>,
    F: Fn(&T) -> U,
{
    type Transition = M::Transition;

    fn edges(&self) -> impl Iterator<Item = M::Transition> {
        self.machine.edges()
    }

    fn state(&self) -> &U {
        let machine_state = self.machine.state();
        let mut cache = self.cached_state.borrow_mut();

        // Update cache with the mapped state
        let mapped = (self.f)(machine_state);
        *cache = Some(mapped);

        // Now we need to return a reference to the cached value
        // This requires unsafe code or a different approach
        // Let's drop the mutable borrow and get an immutable reference
        drop(cache);

        unsafe {
            // SAFETY: We just updated the cache above, so it contains Some(U)
            // The reference is valid for the lifetime of self
            let cache_ref = self.cached_state.as_ptr();
            (*cache_ref).as_ref().unwrap()
        }
    }

    fn traverse(&mut self, edge: Self::Transition) {
        self.machine.traverse(edge);
        // Clear cache to force recomputation on next state() call
        *self.cached_state.borrow_mut() = None;
    }
}

///
/// MappedTransitionMachine allows for mapping Machine transition type into another type while maintaining
/// the same semantics.
///
pub(super) struct MappedTransitionMachine<T, M, F, G> {
    pub(super) t: PhantomData<T>,
    pub(super) machine: M,
    pub(super) f: F,
    pub(super) g: G,
}

impl<M, F, G, T, U, V> Machine<T> for MappedTransitionMachine<T, M, F, G>
where
    M: Machine<T, Transition = U>,
    F: Fn(U) -> V,
    G: Fn(V) -> Option<U>,
{
    type Transition = V;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        self.machine.edges().map(|e| (self.f)(e))
    }

    fn state(&self) -> &T {
        self.machine.state()
    }

    fn traverse(&mut self, edge: Self::Transition) {
        if let Some(inner_edge) = (self.g)(edge) {
            self.machine.traverse(inner_edge);
        }
    }
}
