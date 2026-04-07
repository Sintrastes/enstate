use core::marker::PhantomData;

use super::Machine;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ZippedMachine<T, U, M1, M2, F, V> {
    pub(crate) t: PhantomData<T>,
    pub(crate) u: PhantomData<U>,
    pub(crate) machine1: M1,
    pub(crate) machine2: M2,
    pub(crate) f: F,
    pub(crate) cached_state: Option<V>,
}

impl<M1, M2, F, I: Send, T: Send + Sync + Clone, U: Send + Sync + Clone, V: Send + Sync + Clone>
    Machine<I, V> for ZippedMachine<T, U, M1, M2, F, V>
where
    M1: Machine<I, T>,
    M2: Machine<I, U>,
    I: Clone,
    F: Fn(&T, &U) -> V + Send + Sync + Clone,
{
    fn edges(&self) -> impl Iterator<Item = I> {
        self.machine1.edges().chain(self.machine2.edges())
    }

    fn state(&mut self) -> &V {
        let state1 = self.machine1.state();
        let state2 = self.machine2.state();

        let mapped = (self.f)(state1, state2);
        self.cached_state = Some(mapped);

        &self.cached_state.as_ref().unwrap()
    }

    fn traverse(&mut self, edge: I) {
        self.traverse_ref(&edge);
    }

    fn traverse_ref(&mut self, edge: &I) {
        self.machine1.traverse_ref(edge);
        self.machine2.traverse_ref(edge);
    }
}
