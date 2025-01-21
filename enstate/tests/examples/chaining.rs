use std::marker::PhantomData;

use enstate::machine::Machine;
use enstate::machine::chained::Chainable;

use crate::examples::{
    counter::{Action, counter},
    modal_dialog::{CountDialogAction, ModalAction, ModalResult, modal},
};

#[test]
fn chained_modal_dialog_example() {
    let dialog = || {
        modal()
            .map_actions(
                |x| CountDialogAction::Buttons(x),
                |x| match x {
                    CountDialogAction::Buttons(modal_action) => Some(modal_action),
                    CountDialogAction::Display(_) => None,
                },
            )
            .zip_with(
                counter().map_actions(
                    |x| CountDialogAction::Display(x),
                    |x| match x {
                        CountDialogAction::Buttons(_) => None,
                        CountDialogAction::Display(action) => Some(action),
                    },
                ),
                |dialog_state, count| match dialog_state {
                    Some(f) => Some((f.clone())(count)),
                    None => None,
                },
            )
    };

    let mut machine = dialog().chain(dialog());

    assert_eq!(machine.state(), None);

    // First dialog interactions
    machine.traverse(&CountDialogAction::Display(Action::Increment));
    assert_eq!(machine.state(), None);

    machine.traverse(&CountDialogAction::Buttons(ModalAction::Ok));
    assert_eq!(machine.state(), None); // Still None as we're in second dialog

    // Second dialog interactions
    machine.traverse(&CountDialogAction::Buttons(ModalAction::Ok));
    machine.traverse(&CountDialogAction::Display(Action::Increment));
    assert_eq!(machine.state(), Some(ModalResult::Ok(1)));

    // Test cancellation path
    let mut machine = dialog().chain(dialog());

    machine.traverse(&CountDialogAction::Display(Action::Increment));
    machine.traverse(&CountDialogAction::Buttons(ModalAction::Ok));
    machine.traverse(&CountDialogAction::Buttons(ModalAction::Cancel));
    assert_eq!(machine.state(), Some(ModalResult::Cancelled));
}
