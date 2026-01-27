use core::{iter::empty, marker::PhantomData};

use super::Machine;

#[inline]
pub fn pure<T: Clone, E: Clone>(value: T) -> impl Machine<T, Transition = E> + Clone {
    PureMachine {
        e: PhantomData::<E>,
        value,
    }
}

#[derive(Clone)]
struct PureMachine<T, E> {
    e: PhantomData<E>,
    value: T,
}

impl<T, E> Machine<T> for PureMachine<T, E> {
    type Transition = E;

    fn edges(&self) -> impl Iterator<Item = E> {
        empty()
    }

    fn state(&self) -> &T {
        &self.value
    }

    fn traverse(&mut self, _edge: Self::Transition) {}
}
