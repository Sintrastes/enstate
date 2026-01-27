use crate::machine::Machine;
use core::marker::PhantomData;

/// Creates an accumulating state machine.
///
/// # Arguments
/// - `initial` - The initial state value
/// - `f` - A function that takes the current state and an event, returning the new state
///
/// # Example
/// ```
/// use enstate::machine::accumulator::accumulator;
///
/// // Sum all input values
/// let sum = accumulator(0, |acc: &i32, val: &i32| acc + val);
///
/// // Collect all events into a vector
/// let collector = accumulator(Vec::new(), |acc: &Vec<i32>, item: &i32| {
///     let mut new_acc = acc.clone();
///     new_acc.push(*item);
///     new_acc
/// });
/// ```
pub fn accumulator<T: Clone, E, F: FnMut(&T, &E) -> T>(
    initial: T,
    f: F,
) -> impl Machine<T, Transition = E> {
    Accumulator {
        state: initial,
        f,
        _phantom: PhantomData,
    }
}

struct Accumulator<T: Clone, E, F> {
    state: T,
    f: F,
    _phantom: PhantomData<E>,
}

impl<T: Clone, E, F: FnMut(&T, &E) -> T> Machine<T> for Accumulator<T, E, F> {
    type Transition = E;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        // Accumulator machines accept any input
        core::iter::empty()
    }

    fn state(&self) -> &T {
        &self.state
    }

    fn traverse(&mut self, event: Self::Transition) {
        self.state = (self.f)(&self.state, &event);
    }

    fn traverse_ref(&mut self, event: &Self::Transition) {
        self.state = (self.f)(&self.state, event);
    }
}

#[test]
fn accumulator_works_as_expected() {
    let mut counter = accumulator(0, |count: &i32, _: &()| count + 1);

    let count = counter.state();
    assert_eq!(*count, 0);

    counter.traverse(());

    let count = counter.state();
    assert_eq!(*count, 1);
}
