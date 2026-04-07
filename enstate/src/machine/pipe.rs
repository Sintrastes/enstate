use crate::machine::Machine;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PipedMachine<T, U, M1, M2> {
    pub(crate) t: PhantomData<T>,
    pub(crate) u: PhantomData<U>,
    pub(crate) machine1: M1,
    pub(crate) machine2: M2,
}

impl<M1, M2, I: Send, T: Clone + Send + Sync, U: Send + Sync + Clone> Machine<I, U>
    for PipedMachine<T, U, M1, M2>
where
    M1: Machine<I, T> + Clone,
    M2: Machine<T, U> + Clone,
    I: Clone,
{
    fn edges(&self) -> impl Iterator<Item = I> {
        self.machine1.edges()
    }

    fn state(&mut self) -> &U {
        self.machine2.state()
    }

    fn traverse(&mut self, edge: I) {
        self.traverse_ref(&edge);
    }

    fn traverse_ref(&mut self, edge: &I) {
        self.machine1.traverse_ref(&edge);
        let first_state = self.machine1.state();
        self.machine2.traverse_ref(first_state);
    }
}
