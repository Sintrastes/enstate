#![feature(
    coroutines,
    coroutine_trait,
    trait_alias,
    never_type,
    exhaustive_patterns
)]

use std::{
    marker::PhantomData,
    ops::{Coroutine, CoroutineState},
    pin::pin,
    rc::Rc,
};

use enstate::{
    coroutines::{AsMachine, StateMachine},
    machine::Machine,
};
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

fn counter() -> impl StateMachine<Action, i32> {
    machine!(count, 0, || {
        let action = choose![Action::Increment, Action::Decrement];
        match action {
            Action::Increment => count = count + 1,
            Action::Decrement => count = count - 1,
        }
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum VendingAction {
    InsertCoin,
    SelectItem,
    ReturnChange,
}

impl Default for VendingAction {
    fn default() -> Self {
        Self::InsertCoin
    }
}

fn vending_machine() -> impl StateMachine<VendingAction, u32> {
    machine!(coins, 0, || {
        let action = choose![
            VendingAction::InsertCoin,
            VendingAction::SelectItem,
            VendingAction::ReturnChange
        ];
        match action {
            VendingAction::InsertCoin => coins = coins + 1,
            VendingAction::SelectItem => {
                if coins >= 2 {
                    coins = coins - 2
                }
            }
            VendingAction::ReturnChange => coins = 0,
        }
    })
}

#[test]
fn vending_machine_example() {
    let mut machine = vending_machine();

    let CoroutineState::Yielded(initial) = pin!(&mut machine).resume(VendingAction::InsertCoin);
    assert!(initial.0 == 0);

    let CoroutineState::Yielded(after_insert) =
        pin!(&mut machine).resume(VendingAction::InsertCoin);
    assert!(after_insert.0 == 1);

    let CoroutineState::Yielded(after_select) =
        pin!(&mut machine).resume(VendingAction::SelectItem);
    assert!(after_select.0 == 1); // Not enough coins

    let CoroutineState::Yielded(after_insert2) =
        pin!(&mut machine).resume(VendingAction::InsertCoin);
    assert!(after_insert2.0 == 2);

    let CoroutineState::Yielded(after_purchase) =
        pin!(&mut machine).resume(VendingAction::SelectItem);
    assert!(after_purchase.0 == 0); // Purchase successful
}

#[test]
fn counter_example() {
    let mut machine = counter();

    // The first action is irrelevant given the way we define state machines
    // currently.
    let CoroutineState::Yielded(initial_return) = pin!(&mut machine).resume(Action::Decrement);

    let expected_actions = [Action::Increment, Action::Decrement].as_slice();

    assert!(initial_return == (0, expected_actions));

    // The second action should actually increment the state.
    let CoroutineState::Yielded(second_return) = pin!(&mut machine).resume(Action::Increment);

    eprintln!("{:?}", &second_return);

    assert!(second_return == (1, expected_actions));
}

/// Modal dialog example.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ModalResult<T> {
    Ok(T),
    Cancelled,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum CountDialogAction {
    Buttons(ModalAction),
    Display(Action),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ModalAction {
    Ok,
    Cancelled,
}

impl From<ModalAction> for CountDialogAction {
    fn from(value: ModalAction) -> Self {
        CountDialogAction::Buttons(value)
    }
}

impl From<Action> for CountDialogAction {
    fn from(value: Action) -> Self {
        CountDialogAction::Display(value)
    }
}

impl TryInto<Action> for CountDialogAction {
    type Error = ();

    fn try_into(self) -> Result<Action, Self::Error> {
        match self {
            CountDialogAction::Display(action) => Ok(action),
            _ => Err(()),
        }
    }
}

impl TryInto<ModalAction> for CountDialogAction {
    type Error = ();

    fn try_into(self) -> Result<ModalAction, Self::Error> {
        match self {
            CountDialogAction::Buttons(action) => Ok(action),
            _ => Err(()),
        }
    }
}

impl Default for ModalAction {
    fn default() -> Self {
        ModalAction::Ok
    }
}

///
/// This is a generic machine that can be combined (applicatively)
///  with other machines to construct the state machine for a generic modal
///  dialog.
///
/// This implementation requires a bit overhead (Rc and trait objects), but perhaps
///  there is a way to improve on this.
///
struct ModalState<T> {
    result: Option<T>,
}

impl<T> Machine<Option<fn(T) -> ModalResult<T>>> for ModalState<fn(T) -> ModalResult<T>> {
    type Transition = ModalAction;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        vec![ModalAction::Ok, ModalAction::Cancelled].into_iter()
    }

    fn state(&mut self) -> Option<fn(T) -> ModalResult<T>> {
        self.result.clone()
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        match edge {
            ModalAction::Cancelled => self.result = Some(|_| ModalResult::Cancelled),
            ModalAction::Ok => self.result = Some(|state| ModalResult::Ok(state)),
        }
    }
}

#[test]
fn modal_dialog_example() {
    let contents = AsMachine::new(counter());

    let dialog = ModalState { result: None };

    let mut machine = dialog.zip_with_into(
        PhantomData::<CountDialogAction>,
        contents,
        |dialog_state, count| match dialog_state {
            Some(f) => Some((f.clone())(count)),
            None => None,
        },
    );

    assert_eq!(machine.state(), None);

    machine.traverse(&CountDialogAction::Display(Action::Decrement));

    assert_eq!(machine.state(), None);

    machine.traverse(&CountDialogAction::Buttons(ModalAction::Ok));

    assert_eq!(machine.state(), Some(ModalResult::Ok(-1)));

    let contents = AsMachine::new(counter());

    let dialog = ModalState { result: None };

    let mut machine =
        dialog.zip_with_into(
            PhantomData,
            contents,
            |dialog_state, count| match dialog_state {
                Some(f) => Some((f.clone())(count)),
                None => None,
            },
        );

    assert_eq!(machine.state(), None);

    machine.traverse(&CountDialogAction::Display(Action::Increment));

    assert_eq!(machine.state(), None);

    machine.traverse(&CountDialogAction::Buttons(ModalAction::Ok));

    assert_eq!(machine.state(), Some(ModalResult::Ok(1)));
}
