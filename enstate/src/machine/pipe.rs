use crate::machine::Machine;
use core::marker::PhantomData;

pub(crate) struct PipedMachine<T, U, M1, M2> {
    pub(crate) t: PhantomData<T>,
    pub(crate) u: PhantomData<U>,
    pub(crate) machine1: M1,
    pub(crate) machine2: M2,
}

impl<M1, M2, T: Clone, U> Machine<U> for PipedMachine<T, U, M1, M2>
where
    M1: Machine<T>,
    M2: Machine<U, Transition = T>,
    M1::Transition: Clone,
{
    type Transition = M1::Transition;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        self.machine1.edges()
    }

    fn state(&self) -> &U {
        self.machine2.state()
    }

    fn traverse(&mut self, edge: M1::Transition) {
        self.traverse_ref(&edge);
    }

    fn traverse_ref(&mut self, edge: &Self::Transition) {
        self.machine1.traverse_ref(&edge);
        let first_state = self.machine1.state();
        self.machine2.traverse_ref(first_state);

        todo!()
    }
}
