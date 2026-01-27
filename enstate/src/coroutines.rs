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
pub struct AsMachine<A, S, M> {
    pub a: PhantomData<A>,
    pub state: S,
    pub machine: M,
}

impl<State: Clone, Action: Copy + Default, M: StateMachine<Action, State> + Unpin>
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

impl<State: Clone, Action: Copy + Default, M: StateMachine<Action, State> + Unpin> Machine<State>
    for AsMachine<Action, CoroutineState<(State, &'static [Action]), !>, M>
{
    type Transition = Action;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        let CoroutineState::Yielded(result) = &self.state;
        result.1.into_iter().copied()
    }

    fn state(&self) -> &State {
        match &self.state {
            CoroutineState::Yielded(x) => &x.0,
        }
    }

    fn traverse(&mut self, edge: Self::Transition) {
        self.state = pin!(&mut self.machine).resume(edge);
    }
}

// Note: Needs to be public for macro to work?
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

impl<A: Copy + Default, R, M: ChainStateMachine<A, R> + Unpin> Machine<Option<R>>
    for AsChainMachine<A, R, M>
{
    type Transition = A;

    fn edges(&self) -> impl Iterator<Item = A> {
        self.edges.iter().copied()
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

    fn state(&self) -> &Option<R> {
        &self.result
    }
}
