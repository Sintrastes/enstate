use core::{cell::RefCell, marker::PhantomData};

use super::Machine;

#[derive(Clone)]
pub(crate) struct ZippedMachine<T, U, M1, M2, F, V> {
    pub(crate) t: PhantomData<T>,
    pub(crate) u: PhantomData<U>,
    pub(crate) machine1: M1,
    pub(crate) machine2: M2,
    pub(crate) f: F,
    pub(crate) cached_state: RefCell<Option<V>>,
}

impl<M1, M2, F, T, U, V> Machine<V> for ZippedMachine<T, U, M1, M2, F, V>
where
    M1: Machine<T>,
    M2: Machine<U, Transition = M1::Transition>,
    M1::Transition: Clone,
    F: Fn(&T, &U) -> V,
    V: Clone,
{
    type Transition = M1::Transition;

    fn edges(&self) -> impl Iterator<Item = M1::Transition> {
        self.machine1.edges()
    }

    fn state(&self) -> &V {
        let state1 = self.machine1.state();
        let state2 = self.machine2.state();

        let mut cache = self.cached_state.borrow_mut();
        let mapped = (self.f)(state1, state2);
        *cache = Some(mapped);
        drop(cache);

        unsafe {
            // SAFETY: We just updated the cache above, so it contains Some(V)
            // The reference is valid for the lifetime of self
            let cache_ref = self.cached_state.as_ptr();
            (*cache_ref).as_ref().unwrap()
        }
    }

    fn traverse(&mut self, edge: M1::Transition) {
        self.traverse_ref(&edge);
    }

    fn traverse_ref(&mut self, edge: &M1::Transition) {
        self.machine1.traverse_ref(edge);
        self.machine2.traverse_ref(edge);
        // Clear cache to force recomputation on next state() call
        *self.cached_state.borrow_mut() = None;
    }
}
