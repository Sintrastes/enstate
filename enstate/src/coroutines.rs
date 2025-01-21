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
pub struct AsMachine<A, S, M> {
    pub a: PhantomData<A>,
    pub state: S,
    pub machine: M,
}

impl<State: Clone, Action: Clone + Default, M: StateMachine<Action, State> + Unpin>
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

impl<State: Clone, Action: Clone + Default, M: StateMachine<Action, State> + Unpin> Machine<State>
    for AsMachine<Action, CoroutineState<(State, &'static [Action]), !>, M>
{
    type Transition = Action;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        let CoroutineState::Yielded(result) = &self.state;
        result.1.into_iter().map(|x| x.clone())
    }

    fn state(&mut self) -> State {
        match &self.state {
            CoroutineState::Yielded(x) => x.0.clone(),
        }
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        self.state = pin!(&mut self.machine).resume(edge.clone());
    }
}

pub struct AsChainMachine<A: 'static, R, M> {
    pub a: PhantomData<A>,
    pub state: CoroutineState<&'static [A], R>,
    pub machine: M,
}

impl<A: Clone + Default, R, M: ChainStateMachine<A, R> + Unpin> AsChainMachine<A, R, M> {
    pub fn new(machine: M) -> AsChainMachine<A, R, M> {
        let mut machine = machine;
        let pin = pin!(&mut machine);
        let initial = pin.resume(A::default());

        AsChainMachine {
            a: PhantomData,
            state: initial,
            machine,
        }
    }
}

impl<A: Clone + Default, R: Clone, M: ChainStateMachine<A, R> + Unpin> Machine<Option<R>>
    for AsChainMachine<A, R, M>
{
    type Transition = A;

    fn edges(&self) -> impl Iterator<Item = A> {
        match &self.state {
            CoroutineState::Yielded(actions) => actions.iter().cloned(),
            CoroutineState::Complete(_) => [].iter().cloned(),
        }
    }

    fn traverse(&mut self, edge: &A) {
        self.state = pin!(&mut self.machine).resume(edge.clone());
    }

    fn state(&mut self) -> Option<R> {
        match &self.state {
            CoroutineState::Complete(result) => Some(result.clone()),
            _ => None,
        }
    }
}
