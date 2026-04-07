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
/// let sum = accumulator(0, |acc: &i32, val: &i32| acc + val);
///
/// // Collect all events into a vector
/// let collector = accumulator_mut(Vec::new(), |acc: &mut Vec<i32>, item: &i32| {
///     acc.push(*item);
/// });
/// ```
pub fn accumulator<
    T: Clone + Send + Sync,
    E: Send + Sync + Clone,
    F: FnMut(&T, &E) -> T + Send + Sync + Clone,
>(
    initial: T,
    f: F,
) -> impl Machine<E, T> + Clone {
    Accumulator {
        state: initial,
        f,
        _phantom: PhantomData,
    }
}

#[derive(Debug, Clone, Copy)]
struct Accumulator<T: Clone, E, F> {
    state: T,
    f: F,
    _phantom: PhantomData<E>,
}

impl<T: Clone + Send + Sync, E: Send + Sync + Clone, F: FnMut(&T, &E) -> T + Send + Sync + Clone>
    Machine<E, T> for Accumulator<T, E, F>
{
    fn edges(&self) -> impl Iterator<Item = E> {
        // Accumulator machines accept any input
        core::iter::empty()
    }

    fn state(&mut self) -> &T {
        &self.state
    }

    fn traverse(&mut self, event: E) {
        self.state = (self.f)(&self.state, &event);
    }

    fn traverse_ref(&mut self, event: &E) {
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
