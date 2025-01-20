use enstate::machine::Machine;
use enstate_macros::machine;

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
