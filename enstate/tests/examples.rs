#![feature(
    coroutines,
    coroutine_trait,
    trait_alias,
    never_type,
    exhaustive_patterns
)]

use std::marker::PhantomData;

use enstate::{
    coroutines::ChainStateMachine,
    machine::{ChainMachine, Machine},
};
use enstate_macros::{machine, machine_chain};

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

fn counter() -> impl Machine<i32, Transition = Action> {
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

fn vending_machine() -> impl Machine<u32, Transition = VendingAction> {
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

    assert_eq!(machine.state(), 0);

    machine.traverse(&VendingAction::InsertCoin);
    assert_eq!(machine.state(), 1);

    machine.traverse(&VendingAction::SelectItem);
    assert_eq!(machine.state(), 1); // Not enough coins

    machine.traverse(&VendingAction::InsertCoin);
    assert_eq!(machine.state(), 2);

    machine.traverse(&VendingAction::SelectItem);
    assert_eq!(machine.state(), 0); // Purchase successful
}

#[test]
fn counter_example() {
    let mut machine = counter();

    assert_eq!(machine.state(), 0);

    machine.traverse(&Action::Increment);
    assert_eq!(machine.state(), 1);
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
    Cancel,
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

fn modal<T: Clone>() -> impl ChainMachine<Option<fn(T) -> ModalResult<T>>, Transition = ModalAction>
{
    machine_chain!(|| {
        let action = yield [ModalAction::Ok, ModalAction::Cancel].as_slice();

        match action {
            ModalAction::Ok => |state| ModalResult::Ok(state),
            ModalAction::Cancel => |_| ModalResult::Cancelled,
        }
    })
}

#[test]
fn modal_dialog_example() {
    let contents = counter();

    let dialog = modal();

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

    let contents = counter();

    let dialog = modal();

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
