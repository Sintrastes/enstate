use core::{iter::empty, marker::PhantomData};

use super::Machine;

#[inline]
pub fn pure<T: Clone + Send + Sync, E: Clone + Send + Sync>(
    value: T,
) -> impl Machine<E, T> + Send + Clone {
    PureMachine {
        e: PhantomData::<E>,
        value,
    }
}

#[derive(Debug, Clone, Copy)]
struct PureMachine<T, E> {
    e: PhantomData<E>,
    value: T,
}

impl<T: Send + Sync + Clone, E: Send + Sync + Clone> Machine<E, T> for PureMachine<T, E> {
    fn edges(&self) -> impl Iterator<Item = E> {
        empty()
    }

    fn state(&mut self) -> &T {
        &self.value
    }

    fn traverse(&mut self, _edge: E) {}

    fn traverse_ref(&mut self, _edge: &E) {}
}
