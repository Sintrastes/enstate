use core::marker::PhantomData;

use super::Machine;

pub struct ZippedMachine<T, U, M1, M2, F> {
    pub(crate) t: PhantomData<T>,
    pub(crate) u: PhantomData<U>,
    pub(crate) machine1: M1,
    pub(crate) machine2: M2,
    pub(crate) f: F,
}

impl<M1, M2, F, T, U, V> Machine<V> for ZippedMachine<T, U, M1, M2, F>
where
    M1: Machine<T>,
    M2: Machine<U, Transition = M1::Transition>,
    F: FnMut(T, U) -> V,
{
    type Transition = M1::Transition;

    fn edges(&self) -> impl Iterator<Item = M1::Transition> {
        self.machine1.edges()
    }

    fn state(&mut self) -> V {
        let state1 = self.machine1.state();

        let state2 = self.machine2.state();

        (self.f)(state1, state2)
    }

    fn traverse(&mut self, edge: &M1::Transition) {
        self.machine1.traverse(&edge);
        self.machine2.traverse(&edge);
    }
}
