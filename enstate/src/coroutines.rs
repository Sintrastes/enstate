extern crate alloc;

use alloc::vec::Vec;
use core::{
    marker::PhantomData,
    ops::{Coroutine, CoroutineState},
    pin::pin,
};

use crate::machine::Machine;

///
/// Trait alias used to facilitate constructing machines with Rust coroutines
///  on nightly.
///
/// Best used together with the machine!() macro.
///
pub trait StateMachine<Action: 'static, State> =
    Coroutine<Action, Yield = (State, &'static [Action]), Return = !>;

pub trait ChainStateMachine<Action: 'static, Result> =
    Coroutine<Action, Yield = &'static [Action], Return = Result>;

///
/// Struct used to treat a coroutine state machine as a machine.
///
// Note: Needs to be public for machine macro to work?
#[derive(Clone)]
pub struct AsMachine<A, S, M> {
    pub a: PhantomData<A>,
    pub state: S,
    pub machine: M,
}

impl<State, Action: Copy + Default, M: StateMachine<Action, State> + Unpin>
    AsMachine<Action, CoroutineState<(State, &'static [Action]), !>, M>
{
    pub fn new(machine: M) -> AsMachine<Action, CoroutineState<(State, &'static [Action]), !>, M> {
        let mut machine = machine;
        let pin = pin!(&mut machine);
        let initial = pin.resume(Action::default());

        AsMachine {
            a: PhantomData,
            state: initial,
            machine,
        }
    }
}

impl<
    State: Send + Sync + Clone,
    Action: Copy + Send + Sync,
    M: StateMachine<Action, State> + Unpin + Send + Sync + Clone,
> Machine<Action, State> for AsMachine<Action, CoroutineState<(State, &'static [Action]), !>, M>
{
    fn edges(&self) -> impl Iterator<Item = Action> {
        let CoroutineState::Yielded(result) = &self.state;
        result.1.into_iter().copied()
    }

    fn state(&mut self) -> &State {
        match &self.state {
            CoroutineState::Yielded(x) => &x.0,
        }
    }

    fn traverse_ref(&mut self, edge: &Action) {
        self.traverse(edge.clone());
    }

    fn traverse(&mut self, edge: Action) {
        self.state = pin!(&mut self.machine).resume(edge);
    }
}

// Note: Needs to be public for macro to work?
#[derive(Clone)]
pub struct AsChainMachine<A: 'static, R, M> {
    pub a: PhantomData<A>,
    pub edges: &'static [A],
    pub result: Option<R>,
    pub machine: M,
}

impl<A: Copy + Default, R, M: ChainStateMachine<A, R> + Unpin> AsChainMachine<A, R, M> {
    pub fn new(machine: M) -> AsChainMachine<A, R, M> {
        let mut machine = machine;
        let pin = pin!(&mut machine);
        let initial = pin.resume(A::default());

        let (edges, result) = match initial {
            CoroutineState::Yielded(actions) => (actions, None),
            CoroutineState::Complete(r) => (&[] as &[A], Some(r)),
        };

        AsChainMachine {
            a: PhantomData,
            edges,
            result,
            machine,
        }
    }
}

impl<
    A: Copy + Default + Send + Sync,
    R: Send + Sync + Clone,
    M: ChainStateMachine<A, R> + Unpin + Send + Sync + Clone,
> Machine<A, Option<R>> for AsChainMachine<A, R, M>
{
    fn edges(&self) -> impl Iterator<Item = A> {
        self.edges.iter().copied()
    }

    fn traverse_ref(&mut self, edge: &A) {
        self.traverse(edge.clone());
    }

    fn traverse(&mut self, edge: A) {
        let new_state = pin!(&mut self.machine).resume(edge);

        match new_state {
            CoroutineState::Yielded(actions) => {
                self.edges = actions;
                self.result = None;
            }
            CoroutineState::Complete(r) => {
                self.edges = &[];
                self.result = Some(r);
            }
        }
    }

    fn state(&mut self) -> &Option<R> {
        &self.result
    }
}

///
/// A wrapper around a coroutine-based machine that implements Clone by
/// storing a factory function and replaying the event history into a
/// fresh coroutine on each clone.
///
pub struct ReplayMachine<A: 'static, S, M, F> {
    factory: F,
    history: Vec<A>,
    current: AsMachine<A, CoroutineState<(S, &'static [A]), !>, M>,
}

impl<
    A: 'static + Copy + Default,
    S: Send + Sync + Clone,
    M: StateMachine<A, S> + Unpin,
    F: Fn() -> M,
> ReplayMachine<A, S, M, F>
{
    pub fn new(factory: F) -> Self {
        let current = AsMachine::new(factory());
        ReplayMachine {
            factory,
            history: Vec::new(),
            current,
        }
    }
}

impl<A, S, M, F> Clone for ReplayMachine<A, S, M, F>
where
    A: 'static + Copy + Default,
    S: Send + Sync + Clone,
    M: StateMachine<A, S> + Unpin,
    F: Fn() -> M + Clone,
{
    fn clone(&self) -> Self {
        let mut current = AsMachine::new((self.factory)());
        for &event in &self.history {
            current.state = pin!(&mut current.machine).resume(event);
        }
        ReplayMachine {
            factory: self.factory.clone(),
            history: self.history.clone(),
            current,
        }
    }
}

impl<A, S, M, F> Machine<A, S> for ReplayMachine<A, S, M, F>
where
    A: 'static + Copy + Default + Send + Sync,
    S: Send + Sync + Clone,
    M: StateMachine<A, S> + Unpin + Send + Sync,
    F: Fn() -> M + Clone + Send + Sync,
{
    fn edges(&self) -> impl Iterator<Item = A> {
        let CoroutineState::Yielded(result) = &self.current.state;
        result.1.into_iter().copied()
    }

    fn state(&mut self) -> &S {
        match &self.current.state {
            CoroutineState::Yielded(x) => &x.0,
        }
    }

    fn traverse(&mut self, edge: A) {
        self.history.push(edge);
        self.current.state = pin!(&mut self.current.machine).resume(edge);
    }

    fn traverse_ref(&mut self, edge: &A) {
        self.traverse(*edge);
    }
}

///
/// A wrapper around a coroutine-based chain machine that implements Clone
/// by storing a factory function and replaying the event history into a
/// fresh coroutine on each clone.
///
pub struct ReplayChainMachine<A: 'static, R, M, F> {
    factory: F,
    history: Vec<A>,
    current: AsChainMachine<A, R, M>,
}

impl<
    A: 'static + Copy + Default,
    R,
    M: ChainStateMachine<A, R> + Unpin,
    F: Fn() -> M,
> ReplayChainMachine<A, R, M, F>
{
    pub fn new(factory: F) -> Self {
        let current = AsChainMachine::new(factory());
        ReplayChainMachine {
            factory,
            history: Vec::new(),
            current,
        }
    }
}

impl<A, R, M, F> Clone for ReplayChainMachine<A, R, M, F>
where
    A: 'static + Copy + Default,
    R: Clone,
    M: ChainStateMachine<A, R> + Unpin,
    F: Fn() -> M + Clone,
{
    fn clone(&self) -> Self {
        let mut current = AsChainMachine::new((self.factory)());
        for &event in &self.history {
            if current.result.is_some() {
                break;
            }
            match pin!(&mut current.machine).resume(event) {
                CoroutineState::Yielded(actions) => {
                    current.edges = actions;
                    current.result = None;
                }
                CoroutineState::Complete(r) => {
                    current.edges = &[];
                    current.result = Some(r);
                }
            }
        }
        ReplayChainMachine {
            factory: self.factory.clone(),
            history: self.history.clone(),
            current,
        }
    }
}

impl<A, R, M, F> Machine<A, Option<R>> for ReplayChainMachine<A, R, M, F>
where
    A: 'static + Copy + Default + Send + Sync,
    R: Send + Sync + Clone,
    M: ChainStateMachine<A, R> + Unpin + Send + Sync,
    F: Fn() -> M + Clone + Send + Sync,
{
    fn edges(&self) -> impl Iterator<Item = A> {
        self.current.edges.iter().copied()
    }

    fn state(&mut self) -> &Option<R> {
        &self.current.result
    }

    fn traverse(&mut self, edge: A) {
        self.history.push(edge);
        match pin!(&mut self.current.machine).resume(edge) {
            CoroutineState::Yielded(actions) => {
                self.current.edges = actions;
                self.current.result = None;
            }
            CoroutineState::Complete(r) => {
                self.current.edges = &[];
                self.current.result = Some(r);
            }
        }
    }

    fn traverse_ref(&mut self, edge: &A) {
        self.traverse(*edge);
    }
}
