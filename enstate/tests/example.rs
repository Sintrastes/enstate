#![feature(
    coroutines,
    coroutine_trait,
    trait_alias,
    never_type,
    exhaustive_patterns
)]

use std::{
    ops::{Coroutine, CoroutineState},
    pin::pin,
};

use enstate::coroutines::StateMachine;
use enstate_macros::machine;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Action {
    Increment,
    Decrement,
}

impl Default for Action {
    fn default() -> Self {
        Self::Increment
    }
}

fn example() -> impl StateMachine<Action, i32> {
    machine!(count, 0, || {
        let action = choose![Action::Increment, Action::Decrement];
        match action {
            Action::Increment => count = count + 1,
            Action::Decrement => count = count - 1,
        }
    })
}

#[test]
fn test_example_machine() {
    let mut machine = example();

    // The first action is irrelevant given the way we define state machines
    // currently.
    let initial_return = pin!(&mut machine).resume(Action::Decrement);

    let expected_actions = [Action::Increment, Action::Decrement].as_slice();

    assert!(initial_return == CoroutineState::Yielded((0, expected_actions)));

    // The second action should actually increment the state.
    let second_return = pin!(&mut machine).resume(Action::Increment);

    eprintln!("{:?}", &second_return);

    assert!(second_return == CoroutineState::Yielded((1, expected_actions)));
}
