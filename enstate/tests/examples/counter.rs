use enstate::machine::Machine;
use enstate_macros::machine;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Increment,
    Decrement,
}

impl Default for Action {
    fn default() -> Self {
        Self::Increment
    }
}

pub fn counter() -> impl Machine<i32, Transition = Action> {
    machine!(count, 0, || {
        let action = choose![Action::Increment, Action::Decrement];
        match action {
            Action::Increment => count = count + 1,
            Action::Decrement => count = count - 1,
        }
    })
}

#[test]
fn counter_example() {
    let mut machine = counter();

    assert_eq!(machine.state(), 0);

    machine.traverse(&Action::Increment);
    assert_eq!(machine.state(), 1);
}
