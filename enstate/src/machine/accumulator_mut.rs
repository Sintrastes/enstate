use crate::machine::{Machine, event::Event};
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
/// let collector = accumulator_mut(Vec::new(), |acc: &mut Vec<i32>, item: &i32| {
///     acc.push(*item);
/// });
/// ```
pub fn accumulator_mut<
    T: Clone + Send + Sync,
    E: Send + Sync + Clone,
    F: FnMut(&mut T, &E) -> () + Send + Sync + Clone,
>(
    initial: T,
    f: F,
) -> impl Machine<E, T> + Clone {
    AccumulatorMut {
        state: initial,
        f,
        _phantom: PhantomData,
    }
}

pub fn foldp<
    T: Clone + Send + Sync,
    I: Send + Clone,
    E: Send + Clone + Sync,
    F: FnMut(&mut T, &E) -> () + Send + Sync + Clone + 'static,
    M,
>(
    initial: T,
    event: M,
    mut f: F,
) -> impl Machine<I, T> + Clone
where
    M: Machine<I, Event<E>> + Clone + Sync,
{
    event.pipe(accumulator_mut(
        initial,
        move |state, event: &Event<E>| match event {
            Event::Event(event) => f(state, event),
            _ => (),
        },
    ))
}

#[derive(Debug, Clone, Copy)]
struct AccumulatorMut<T: Clone, E, F> {
    state: T,
    f: F,
    _phantom: PhantomData<E>,
}

impl<
    T: Clone + Send + Sync,
    E: Send + Sync + Clone,
    F: FnMut(&mut T, &E) -> () + Send + Sync + Clone,
> Machine<E, T> for AccumulatorMut<T, E, F>
{
    fn edges(&self) -> impl Iterator<Item = E> {
        // Accumulator machines accept any input
        core::iter::empty()
    }

    fn state(&mut self) -> &T {
        &self.state
    }

    fn traverse(&mut self, event: E) {
        (self.f)(&mut self.state, &event);
    }

    fn traverse_ref(&mut self, event: &E) {
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
