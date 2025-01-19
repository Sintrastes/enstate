use std::{
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
