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
struct AsMachine<A, S, M> {
    a: PhantomData<A>,
    state: S,
    machine: M,
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
        match &mut self.state {
            CoroutineState::Yielded(x) => x.0.clone(),
        }
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        self.state = pin!(&mut self.machine).resume(edge.clone());
    }
}
