use enstate::machine::Machine;
use enstate::machine::chained::{Chainable, FlatMappable};
use enstate::optics::Prism;

use crate::examples::{
    counter::{Action, counter},
    modal_dialog::{CountDialogAction, ModalAction, ModalResult, modal},
};

#[test]
fn chained_modal_dialog_example() {
    let modal_prism = Prism::new(
        |x: &CountDialogAction| match x {
            CountDialogAction::Buttons(modal_action) => Some(*modal_action),
            CountDialogAction::Display(_) => None,
        },
        |x: &ModalAction| CountDialogAction::Buttons(*x),
    );

    let counter_prism = Prism::new(
        |x: &CountDialogAction| match x {
            CountDialogAction::Buttons(_) => None,
            CountDialogAction::Display(action) => Some(*action),
        },
        |x: &Action| CountDialogAction::Display(*x),
    );

    let dialog = || {
        modal().lift(&modal_prism).zip_with(
            counter().lift(&counter_prism),
            |dialog_state, count| match dialog_state {
                Some(f) => Some((f)(*count)),
                None => None,
            },
        )
    };

    let mut machine = dialog().chain(dialog());

    assert_eq!(*machine.state(), None);

    // First dialog interactions
    machine.traverse(CountDialogAction::Display(Action::Increment));
    assert_eq!(*machine.state(), None);

    machine.traverse(CountDialogAction::Buttons(ModalAction::Ok));
    assert_eq!(*machine.state(), None); // Still None as we're still in the first dialog

    // Second dialog interactions
    machine.traverse(CountDialogAction::Buttons(ModalAction::Ok));
    machine.traverse(CountDialogAction::Display(Action::Increment));
    assert_eq!(*machine.state(), Some(ModalResult::Ok(1)));

    // Test cancellation path
    let mut machine = dialog().chain(dialog());

    machine.traverse(CountDialogAction::Display(Action::Increment));
    machine.traverse(CountDialogAction::Buttons(ModalAction::Ok));
    machine.traverse(CountDialogAction::Buttons(ModalAction::Cancel));
    assert_eq!(*machine.state(), Some(ModalResult::Cancelled));
}

#[test]
fn flat_map_with_pure_machines() {
    // Simple test demonstrating flat_map works with pure machines
    use enstate::machine::constant::pure;

    // Create a machine that returns Some(5), then flat_maps to double it
    let mut machine = pure::<Option<i32>, Action>(Some(5))
        .flat_map(|x: &i32| pure::<Option<i32>, Action>(Some(x * 2)));

    assert_eq!(*machine.state(), Some(10));

    // Test with None - flat_map won't be called since state is None
    let mut machine2 = pure::<Option<i32>, Action>(None)
        .flat_map(|x: &i32| pure::<Option<i32>, Action>(Some(x * 2)));

    assert_eq!(*machine2.state(), None);
}
