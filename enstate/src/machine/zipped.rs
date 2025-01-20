use std::marker::PhantomData;

use super::Machine;

pub struct ZippedMachine<E, T, U, M1, M2, F> {
    pub(crate) e: PhantomData<E>,
    pub(crate) t: PhantomData<T>,
    pub(crate) u: PhantomData<U>,
    pub(crate) machine1: M1,
    pub(crate) machine2: M2,
    pub(crate) f: F,
}

impl<E, M1, M2, F, T, U, V> Machine<V> for ZippedMachine<E, T, U, M1, M2, F>
where
    M1: Machine<T>,
    M2: Machine<U>,
    M1::Transition: Into<E>,
    M2::Transition: Into<E>,
    E: Clone,
    E: TryInto<M1::Transition>,
    E: TryInto<M2::Transition>,
    F: FnMut(T, U) -> V,
{
    type Transition = E;

    fn edges(&self) -> impl Iterator<Item = E> {
        self.machine1.edges().map(|x| x.into())
    }

    fn state(&mut self) -> V {
        let state1 = self.machine1.state();

        let state2 = self.machine2.state();

        (self.f)(state1, state2)
    }

    fn traverse(&mut self, edge: &E) {
        let result: Result<M1::Transition, _> = edge.clone().try_into();
        if let Ok(edge) = result {
            self.machine1.traverse(&edge);
        }

        let result: Result<M2::Transition, _> = edge.clone().try_into();
        if let Ok(edge) = result {
            self.machine2.traverse(&edge);
        }
    }
}
