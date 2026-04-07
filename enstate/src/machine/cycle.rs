use core::marker::PhantomData;

use crate::machine::{Machine, event::Event};

/// Feed the second component of a machine's output back as the second component
/// of its input, creating a loop with an explicit initial feedback value.
///
/// The machine `M` takes `(I, L)` and produces `(O, L)`. On each traversal the
/// previous output `L` is fed back as the next input `L`, starting from `initial`.
pub fn cycle<I, O, L, M>(initial: L, machine: M) -> impl Machine<I, O> + Clone
where
    M: Machine<(I, L), (O, L)> + Clone,
    I: Clone + Send + Sync,
    O: Clone + Send + Sync,
    L: Clone + Send + Sync,
{
    CycleMachine {
        i: PhantomData,
        o: PhantomData,
        machine,
        loop_state: initial,
    }
}

#[derive(Clone)]
struct CycleMachine<I, O, L, M> {
    i: PhantomData<I>,
    o: PhantomData<O>,
    machine: M,
    loop_state: L,
}

impl<I, O, L, M> Machine<I, O> for CycleMachine<I, O, L, M>
where
    M: Machine<(I, L), (O, L)> + Clone,
    I: Clone + Send + Sync,
    O: Clone + Send + Sync,
    L: Clone + Send + Sync,
{
    fn edges(&self) -> impl Iterator<Item = I>
    where
        Self: Sized,
    {
        self.machine.edges().map(|x| x.0)
    }

    fn state(&mut self) -> &O {
        &self.machine.state().0
    }

    fn traverse(&mut self, edge: I) {
        self.machine.traverse((edge, self.loop_state.clone()));
        self.loop_state = self.machine.state().1.clone();
    }

    fn traverse_ref(&mut self, edge: &I) {
        self.machine
            .traverse_ref(&(edge.clone(), self.loop_state.clone()));
        self.loop_state = self.machine.state().1.clone();
    }
}

pub fn event_loop<
    A: Send + Sync + Clone,
    C: Send + Sync + Clone,
    D: Send + Sync,
    M: Machine<(A, Event<C>), (D, Event<C>)>,
>(
    machine: M,
) -> impl Machine<A, D> {
    EventLoopMachine {
        a: PhantomData,
        d: PhantomData,
        machine,
        loop_state: Event::NoEvent,
    }
}

struct EventLoopMachine<A, C, D, M> {
    a: PhantomData<A>,
    d: PhantomData<D>,
    machine: M,
    loop_state: Event<C>,
}

impl<
    A: Send + Sync + Clone,
    C: Send + Sync + Clone,
    D: Send + Sync,
    M: Machine<(A, Event<C>), (D, Event<C>)>,
> Machine<A, D> for EventLoopMachine<A, C, D, M>
{
    fn edges(&self) -> impl Iterator<Item = A>
    where
        Self: Sized,
    {
        self.machine.edges().map(|x| x.0)
    }

    fn state(&mut self) -> &D {
        &self.machine.state().0
    }

    fn traverse(&mut self, edge: A) {
        self.machine.traverse((edge, self.loop_state.clone()));
        self.loop_state = self.machine.state().1.clone();
    }

    fn traverse_ref(&mut self, edge: &A) {
        self.machine
            .traverse_ref(&(edge.clone(), self.loop_state.clone()));
        self.loop_state = self.machine.state().1.clone();
    }
}
