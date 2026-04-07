use core::marker::PhantomData;

use super::Machine;
///
/// Trait for machines that can be chainable, usually of the
///  form Machine<Option<T>>.
///
pub trait Chainable<I: Send, T: Send + Sync>: Machine<I, T> {
    type Result<X: Send + Sync>: Send + Sync;

    fn chain<U: Clone + Send + Sync, M2>(self, next: M2) -> impl Machine<I, Self::Result<U>>
    where
        M2: Machine<I, Self::Result<U>>,
        I: Clone + Send + Sync;
}

pub trait JoinMachine<M1, I: Send, T: Send + Sync>
where
    M1: Machine<I, T>,
{
    fn join(self) -> impl Machine<I, T> + Clone
    where
        I: Clone;
}

pub trait FlatMappable<I: Send + Clone, AX: Send + Sync, F, BX: Send + Sync>:
    Machine<I, AX>
{
    fn flat_map(self, f: F) -> impl Machine<I, BX> + Clone;
}

impl<
    I: Send + Sync,
    A: Clone + Send + Sync,
    B: Clone + Send + Sync,
    M1: Machine<I, Option<A>> + Clone + Sync,
    M2: Machine<I, Option<B>> + Clone + Sync,
    F: Fn(&A) -> M2 + Send + Sync + Clone,
> FlatMappable<I, Option<A>, F, Option<B>> for M1
where
    I: Clone,
{
    fn flat_map(self, f: F) -> impl Machine<I, Option<B>> + Clone {
        self.map(move |x: &Option<A>| x.as_ref().map(|a| f(a)))
            .join()
    }
}

#[derive(Clone)]
enum JoinedMachineState<M1, M2> {
    First(M1),
    Second(M2),
}

#[derive(Clone)]
enum JoinedMachineIterator<I1, I2> {
    First(I1),
    Second(I2),
}

impl<I1: Iterator, I2: Iterator<Item = I1::Item>> Iterator for JoinedMachineIterator<I1, I2> {
    type Item = I1::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            JoinedMachineIterator::First(x) => x.next(),
            JoinedMachineIterator::Second(y) => y.next(),
        }
    }
}

#[derive(Clone)]
struct JoinedMachine<I, T, M1, M2> {
    i: PhantomData<I>,
    t: PhantomData<T>,
    state: JoinedMachineState<M1, M2>,
}

impl<I: Send + Sync, T: Clone + Send + Sync, M2, M1> Machine<I, Option<T>>
    for JoinedMachine<I, T, M1, M2>
where
    M1: Machine<I, Option<M2>> + Sync,
    M2: Machine<I, Option<T>> + Clone + Sync,
    I: Clone,
{
    fn edges(&self) -> impl Iterator<Item = I> {
        match &self.state {
            JoinedMachineState::First(i1) => JoinedMachineIterator::First(i1.edges()),
            JoinedMachineState::Second(i2) => JoinedMachineIterator::Second(i2.edges()),
        }
    }

    fn state(&mut self) -> &Option<T> {
        match &mut self.state {
            JoinedMachineState::First(_) => &None,
            JoinedMachineState::Second(m2) => m2.state(),
        }
    }

    fn traverse(&mut self, edge: I) {
        self.traverse_ref(&edge);
    }

    fn traverse_ref(&mut self, edge: &I) {
        match &mut self.state {
            JoinedMachineState::First(m1) => m1.traverse_ref(edge),
            JoinedMachineState::Second(m2) => m2.traverse_ref(edge),
        };

        // Try to get the second machine from the first machine's state
        match &mut self.state {
            JoinedMachineState::First(m1) => {
                if let Some(m2) = (*m1.state()).clone() {
                    self.state = JoinedMachineState::Second(m2);
                }
            }
            _ => {}
        };
    }
}

#[derive(Clone)]
struct ChainedMachine<T, U, M1, M2> {
    t: PhantomData<T>,
    u: PhantomData<U>,
    in_second_machine: bool,
    machine1: M1,
    machine2: M2,
    cached_state: Option<Option<U>>,
}

impl<T, U, M1, M2> ChainedMachine<T, U, M1, M2> {
    fn new(machine1: M1, machine2: M2) -> Self {
        ChainedMachine {
            t: PhantomData,
            u: PhantomData,
            in_second_machine: false,
            machine1,
            machine2,
            cached_state: None,
        }
    }
}

struct ChainedMachineIterator<I1, I2> {
    in_second_machine: bool,
    iterator1: I1,
    iterator2: I2,
}

impl<T, I1: Iterator<Item = T>, I2: Iterator<Item = T>> Iterator
    for ChainedMachineIterator<I1, I2>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.in_second_machine {
            self.iterator2.next()
        } else {
            self.iterator1.next()
        }
    }
}

impl<
    I: Clone + Send + Sync,
    T: Clone + Send + Sync,
    U: Clone + Send + Sync,
    M1: Machine<I, Option<T>> + Sync,
    M2: Machine<I, Option<U>>,
> Machine<I, Option<U>> for ChainedMachine<T, U, M1, M2>
{
    fn edges(&self) -> impl Iterator<Item = I> {
        ChainedMachineIterator {
            in_second_machine: self.in_second_machine,
            iterator1: self.machine1.edges(),
            iterator2: self.machine2.edges(),
        }
    }

    fn state(&mut self) -> &Option<U> {
        let state_value = if self.in_second_machine {
            (*self.machine2.state()).clone()
        } else {
            None
        };

        self.cached_state = Some(state_value);

        &self.cached_state.as_ref().unwrap()
    }

    fn traverse(&mut self, edge: I) {
        self.traverse_ref(&edge);
    }

    fn traverse_ref(&mut self, edge: &I) {
        if self.in_second_machine {
            self.machine2.traverse_ref(edge);
        } else {
            self.machine1.traverse_ref(edge);

            let new_state = (*self.machine1.state()).clone();

            if let Some(_) = new_state {
                self.in_second_machine = true;
            }
        }
    }
}

impl<I: Send + Sync, T: Clone + Send + Sync, M1: Machine<I, Option<T>> + Sync>
    Chainable<I, Option<T>> for M1
{
    type Result<X: Send + Sync> = Option<X>;

    #[inline]
    fn chain<U: Clone + Send + Sync, M2>(self, next: M2) -> impl Machine<I, Option<U>>
    where
        M2: Machine<I, Option<U>>,
        I: Clone,
    {
        ChainedMachine::new(self, next)
    }
}

impl<
    I: Send + Sync,
    T: Clone + Send + Sync,
    M1: Machine<I, Option<T>> + Clone + Sync,
    M2: Machine<I, Option<M1>> + Clone + Sync,
> JoinMachine<M1, I, Option<T>> for M2
where
    I: Clone,
{
    #[inline]
    fn join(mut self) -> impl Machine<I, Option<T>> + Clone
    where
        Self: Machine<I, Option<M1>>,
        I: Clone,
    {
        // Check if the machine is already complete
        let initial_state = if let Some(m1) = self.state() {
            JoinedMachineState::Second(m1.clone())
        } else {
            JoinedMachineState::First(self)
        };

        JoinedMachine {
            i: PhantomData,
            t: PhantomData,
            state: initial_state,
        }
    }
}
