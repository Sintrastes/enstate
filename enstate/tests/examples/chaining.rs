use enstate::machine::chained::{Chainable, pure};
use enstate::machine::{Machine, chained::FlatMappable};

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

#[test]
fn flat_mapped_modal_dialog_example() {
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

    let mut machine = dialog().flat_map(|first_result| match first_result {
        ModalResult::Ok(first_count) => dialog().map(move |second_result| match second_result {
            Some(ModalResult::Ok(second_count)) => {
                Some(ModalResult::Ok(first_count + second_count))
            }
            Some(ModalResult::Cancelled) => Some(ModalResult::Cancelled),
            None => None,
        }),
        ModalResult::Cancelled => todo!(), // pure(Some(ModalResult::Cancelled)),
    });

    assert_eq!(machine.state(), None);

    // First dialog interactions
    machine.traverse(&CountDialogAction::Display(Action::Increment));
    assert_eq!(machine.state(), None);

    machine.traverse(&CountDialogAction::Buttons(ModalAction::Ok));
    assert_eq!(machine.state(), None);

    // Second dialog interactions
    machine.traverse(&CountDialogAction::Display(Action::Increment));
    machine.traverse(&CountDialogAction::Display(Action::Increment));
    machine.traverse(&CountDialogAction::Buttons(ModalAction::Ok));
    assert_eq!(machine.state(), Some(ModalResult::Ok(3)));

    // TODO: This is currently not working due to the signature of flat_map.
    // Test cancellation path
    // let mut machine = dialog().flat_map(|first_result| match first_result {
    //     ModalResult::Ok(first_count) => dialog().map(move |second_result| match second_result {
    //         Some(ModalResult::Ok(second_count)) => {
    //             Some(ModalResult::Ok(first_count + second_count))
    //         }
    //         Some(ModalResult::Cancelled) => Some(ModalResult::Cancelled),
    //         None => None,
    //     }),
    //     ModalResult::Cancelled => todo!(), //pure(Some(ModalResult::Cancelled)),
    // });

    // machine.traverse(&CountDialogAction::Display(Action::Increment));
    // machine.traverse(&CountDialogAction::Buttons(ModalAction::Cancel));
    // assert_eq!(machine.state(), Some(ModalResult::Cancelled));
}
