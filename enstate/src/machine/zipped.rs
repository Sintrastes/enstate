use std::marker::PhantomData;

use super::Machine;
use core::fmt::Debug;

pub struct ZippedMachine<E, T, U, M1, M2, F> {
    pub(crate) e: PhantomData<E>,
    pub(crate) t: PhantomData<T>,
    pub(crate) u: PhantomData<U>,
    pub(crate) machine1: M1,
    pub(crate) machine2: M2,
    pub(crate) f: F,
}

impl<E, M1, M2, F, T, U: Debug, V> Machine<V> for ZippedMachine<E, T, U, M1, M2, F>
where
    M1: Machine<T>,
    M2: Machine<U>,
    M1::Transition: Into<E>,
    M2::Transition: Into<E>,
    M1::Transition: Debug,
    M2::Transition: Debug,
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

        //eprintln!("State 1 is: {:?}", state1);

        let state2 = self.machine2.state();

        eprintln!("State 2 is: {:?}", state2);

        (self.f)(state1, state2)
    }

    fn traverse(&mut self, edge: &E) {
        let result: Result<M1::Transition, _> = edge.clone().try_into();
        if let Ok(edge) = result {
            eprintln!("Traversing machine 1: {:?}", &edge);
            self.machine1.traverse(&edge);
        }

        let result: Result<M2::Transition, _> = edge.clone().try_into();
        if let Ok(edge) = result {
            eprintln!("Traversing machine 2: {:?}", &edge);
            self.machine2.traverse(&edge);
        }
    }
}
