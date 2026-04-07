use core::marker::PhantomData;

use super::Machine;

///
/// MappedMachine allows for mapping Machine state type into another type while maintaining
/// the same semantics.
/// T is the input state type from the inner machine
/// U is the output state type after mapping
///
#[derive(Debug, Clone, Copy)]
pub(super) struct MappedMachine<U, M, F, T> {
    pub(super) u: PhantomData<U>,
    pub(super) t: PhantomData<T>,
    pub(super) machine: M,
    pub(super) f: F,
    pub(super) cached_state: Option<U>,
}

impl<M, F, I: Send + Sync, T: Send + Sync + Clone, U: Send + Sync + Clone> Machine<I, U>
    for MappedMachine<U, M, F, T>
where
    M: Machine<I, T>,
    F: Fn(&T) -> U + Send + Sync + Clone,
{
    fn edges(&self) -> impl Iterator<Item = I> {
        self.machine.edges()
    }

    fn state(&mut self) -> &U {
        let machine_state = self.machine.state();

        // Update cache with the mapped state
        let mapped = (self.f)(machine_state);
        self.cached_state = Some(mapped);

        &self.cached_state.as_ref().unwrap()
    }

    fn traverse(&mut self, edge: I) {
        self.machine.traverse_ref(&edge);
    }

    fn traverse_ref(&mut self, edge: &I) {
        self.machine.traverse_ref(&edge);
    }
}

///
/// MappedTransitionMachine allows for mapping Machine transition type into another type while maintaining
/// the same semantics.
///
#[derive(Debug, Clone, Copy)]
pub(super) struct MappedTransitionMachine<T, M, F, G> {
    pub(super) t: PhantomData<T>,
    pub(super) machine: M,
    pub(super) f: F,
    pub(super) g: G,
}

impl<M, F: Send + Sync + Clone, G: Send + Sync + Clone, T: Send + Sync + Clone, U: Send, V: Send>
    Machine<V, T> for MappedTransitionMachine<T, M, F, G>
where
    M: Machine<U, T>,
    F: Fn(&U) -> V,
    G: Fn(&V) -> Option<U>,
{
    fn edges(&self) -> impl Iterator<Item = V> {
        self.machine.edges().map(|e| (self.f)(&e))
    }

    fn state(&mut self) -> &T {
        self.machine.state()
    }

    fn traverse(&mut self, edge: V) {
        self.traverse_ref(&edge);
    }

    fn traverse_ref(&mut self, edge: &V) {
        if let Some(inner_edge) = &mut (self.g)(edge) {
            self.machine.traverse_ref(inner_edge);
        }
    }
}
