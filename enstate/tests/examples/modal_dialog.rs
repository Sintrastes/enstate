use std::marker::PhantomData;

use enstate::machine::Machine;
use enstate::machine::chained::Chainable;
use enstate_macros::machine_chain;

use crate::examples::counter::counter;

use super::counter::Action;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModalResult<T> {
    Ok(T),
    Cancelled,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CountDialogAction {
    Buttons(ModalAction),
    Display(Action),
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ModalAction {
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
pub fn modal<T: Clone>() -> impl Chainable<Option<fn(T) -> ModalResult<T>>, Transition = ModalAction>
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
