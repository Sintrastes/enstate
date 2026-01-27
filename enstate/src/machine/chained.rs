use core::{cell::RefCell, marker::PhantomData};

use super::Machine;

///
/// Trait for machines that can be chainable, usually of the
///  form Machine<Option<T>>.
///
pub trait Chainable<T>: Machine<T> {
    type Result<X>;

    fn chain<U: Clone, M2>(
        self,
        next: M2,
    ) -> impl Machine<Self::Result<U>, Transition = Self::Transition>
    where
        M2: Machine<Self::Result<U>, Transition = Self::Transition>,
        Self::Transition: Clone;
}

pub trait JoinMachine<M1, T>
where
    M1: Machine<T>,
{
    fn join(self) -> impl Machine<T, Transition = M1::Transition>
    where
        M1::Transition: Clone;
}

pub trait FlatMappable<AX, F, BX>: Machine<AX> {
    fn flat_map(self, f: F) -> impl Machine<BX, Transition = Self::Transition>
    where
        Self::Transition: Clone;
}

impl<
    A: Clone,
    B: Clone,
    M1: Machine<Option<A>>,
    M2: Machine<Option<B>, Transition = M1::Transition> + Clone,
    F: Fn(&A) -> M2,
> FlatMappable<Option<A>, F, Option<B>> for M1
where
    M1::Transition: Clone,
{
    fn flat_map(self, f: F) -> impl Machine<Option<B>, Transition = Self::Transition> {
        self.map(move |x: &Option<A>| x.as_ref().map(|a| f(a)))
            .join()
    }
}

enum JoinedMachineState<M1, M2> {
    First(M1),
    Second(M2),
}

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

struct JoinedMachine<T, M1, M2> {
    t: PhantomData<T>,
    state: JoinedMachineState<M1, M2>,
    cached_state: RefCell<Option<Option<T>>>,
}

impl<T: Clone, M2, M1> Machine<Option<T>> for JoinedMachine<T, M1, M2>
where
    M1: Machine<Option<M2>, Transition = M2::Transition>,
    M2: Machine<Option<T>> + Clone,
    M2::Transition: Clone,
{
    type Transition = M2::Transition;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        match &self.state {
            JoinedMachineState::First(i1) => JoinedMachineIterator::First(i1.edges()),
            JoinedMachineState::Second(i2) => JoinedMachineIterator::Second(i2.edges()),
        }
    }

    fn state(&self) -> &Option<T> {
        let mut cache = self.cached_state.borrow_mut();

        let state_value = match &self.state {
            JoinedMachineState::First(_) => None,
            JoinedMachineState::Second(m2) => (*m2.state()).clone(),
        };

        *cache = Some(state_value);
        drop(cache);

        unsafe {
            // SAFETY: We just updated the cache above
            let cache_ref = self.cached_state.as_ptr();
            (*cache_ref).as_ref().unwrap()
        }
    }

    fn traverse(&mut self, edge: Self::Transition) {
        match &mut self.state {
            JoinedMachineState::First(m1) => m1.traverse(edge),
            JoinedMachineState::Second(m2) => m2.traverse(edge),
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

        // Clear cache to force recomputation on next state() call
        *self.cached_state.borrow_mut() = None;
    }
}

struct ChainedMachine<T, U, M1, M2> {
    t: PhantomData<T>,
    u: PhantomData<U>,
    in_second_machine: bool,
    machine1: M1,
    machine2: M2,
    cached_state: RefCell<Option<Option<U>>>,
}

impl<T, U, M1, M2> ChainedMachine<T, U, M1, M2> {
    fn new(machine1: M1, machine2: M2) -> Self {
        ChainedMachine {
            t: PhantomData,
            u: PhantomData,
            in_second_machine: false,
            machine1,
            machine2,
            cached_state: RefCell::new(None),
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
    T: Clone,
    U: Clone,
    M1: Machine<Option<T>>,
    M2: Machine<Option<U>, Transition = M1::Transition>,
> Machine<Option<U>> for ChainedMachine<T, U, M1, M2>
where
    M1::Transition: Clone,
{
    type Transition = M2::Transition;

    fn edges(&self) -> impl Iterator<Item = M2::Transition> {
        ChainedMachineIterator {
            in_second_machine: self.in_second_machine,
            iterator1: self.machine1.edges(),
            iterator2: self.machine2.edges(),
        }
    }

    fn state(&self) -> &Option<U> {
        let mut cache = self.cached_state.borrow_mut();

        let state_value = if self.in_second_machine {
            (*self.machine2.state()).clone()
        } else {
            None
        };

        *cache = Some(state_value);
        drop(cache);

        unsafe {
            // SAFETY: We just updated the cache above
            let cache_ref = self.cached_state.as_ptr();
            (*cache_ref).as_ref().unwrap()
        }
    }

    fn traverse(&mut self, edge: Self::Transition) {
        if self.in_second_machine {
            self.machine2.traverse(edge);
        } else {
            self.machine1.traverse(edge);

            let new_state = (*self.machine1.state()).clone();

            if let Some(_) = new_state {
                self.in_second_machine = true;
            }
        }

        // Clear cache to force recomputation on next state() call
        *self.cached_state.borrow_mut() = None;
    }
}

impl<T: Clone, M1: Machine<Option<T>>> Chainable<Option<T>> for M1 {
    type Result<X> = Option<X>;

    #[inline]
    fn chain<U: Clone, M2>(self, next: M2) -> impl Machine<Option<U>, Transition = Self::Transition>
    where
        M2: Machine<Option<U>, Transition = Self::Transition>,
        Self::Transition: Clone,
    {
        ChainedMachine::new(self, next)
    }
}

impl<T: Clone, M1: Machine<Option<T>> + Clone, M2: Machine<Option<M1>, Transition = M1::Transition>>
    JoinMachine<M1, Option<T>> for M2
where
    M1::Transition: Clone,
{
    #[inline]
    fn join(self) -> impl Machine<Option<T>, Transition = M1::Transition>
    where
        Self: Machine<Option<M1>>,
        M1::Transition: Clone,
    {
        // Check if the machine is already complete
        let initial_state = if let Some(m1) = (*self.state()).clone() {
            JoinedMachineState::Second(m1)
        } else {
            JoinedMachineState::First(self)
        };

        JoinedMachine {
            t: PhantomData,
            state: initial_state,
            cached_state: RefCell::new(None),
        }
    }
}
