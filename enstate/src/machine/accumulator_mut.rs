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
/// use enstate::machine::accumulator_mut::accumulator_mut;
///
/// // Sum all input values
/// let sum = accumulator_mut(0, |acc: &mut i32, val: &i32| *acc = *acc + val);
///
/// // Collect all events into a vector
/// let collector = accumulator(Vec::new(), |acc: &Vec<i32>, item: &i32| {
///     let mut new_acc = acc.clone();
///     new_acc.push(*item);
///     new_acc
/// });
/// ```
pub fn accumulator_mut<T: Clone, E, F: FnMut(&mut T, &E) -> ()>(
    initial: T,
    f: F,
) -> impl Machine<T, Transition = E> {
    AccumulatorMut {
        state: initial,
        f,
        _phantom: PhantomData,
    }
}

struct AccumulatorMut<T: Clone, E, F> {
    state: T,
    f: F,
    _phantom: PhantomData<E>,
}

impl<T: Clone, E, F: FnMut(&mut T, &E) -> ()> Machine<T> for AccumulatorMut<T, E, F> {
    type Transition = E;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        // Accumulator machines accept any input
        core::iter::empty()
    }

    fn state(&self) -> &T {
        &self.state
    }

    fn traverse(&mut self, event: Self::Transition) {
        (self.f)(&mut self.state, &event);
    }

    fn traverse_ref(&mut self, event: &Self::Transition) {
        (self.f)(&mut self.state, event);
    }
}

#[test]
fn accumulator_mut_works_as_expected() {
    let mut counter = accumulator_mut(0, |count: &mut i32, _: &()| {
        *count = *count + 1;
    });

    let count = counter.state();
    assert_eq!(*count, 0);

    counter.traverse(());

    let count = counter.state();
    assert_eq!(*count, 1);
}
