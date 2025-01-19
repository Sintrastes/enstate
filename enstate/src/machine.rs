use core::iter::empty;

use core::marker::PhantomData;

use zipped::ZippedMachine;

pub mod zipped;

///
/// Trait for a composable state machine with state of type T.
///
/// State machines have an associated type Transition reprenting
///  the "edges" that can be traversed in a state transition.
///
/// Generally speaking only machines with the same transition type can be composed.
///
pub trait Machine<T>: Sized {
    type Transition;

    ///
    /// Get the possible edges which can be used to transition
    /// out of the current state of the machine.
    ///
    fn edges(&self) -> impl Iterator<Item = Self::Transition>;

    ///
    /// Get the current state of the machine.
    ///
    fn state(&mut self) -> T;

    ///
    /// Traverse along an edge to update the state of the
    ///  machine.
    ///
    /// If the transition is not in the current edges,
    ///  this should be a no-op.
    ///
    fn traverse(&mut self, edge: &Self::Transition);

    ///
    /// Transform the state of a machine by applying a function.
    ///
    #[inline]
    fn map<F, U>(self, f: F) -> MappedMachine<T, Self, F>
    where
        F: FnMut(T) -> U,
    {
        MappedMachine {
            t: PhantomData,
            machine: self,
            f,
        }
    }

    ///
    /// Combine two machines "horizontally", combinding their state with a function.
    ///
    #[inline]
    fn zip_with_into<E, M2, U, G, W: Clone>(
        self,
        _event: PhantomData<E>,
        machine2: M2,
        f: G,
    ) -> ZippedMachine<E, T, U, Self, M2, G>
    where
        M2: Machine<U>,
        // M2::Transition: PartialEq<Self::Transition>,
        Self::Transition: Into<E>,
        M2::Transition: Into<E>,
        E: Clone,
        E: TryInto<Self::Transition>,
        E: TryInto<M2::Transition>,
        G: FnMut(T, U) -> W,
    {
        ZippedMachine {
            e: PhantomData,
            t: PhantomData,
            u: PhantomData,
            machine1: self,
            machine2,
            f,
        }
    }
}

///
/// Trait for machines that can be chainable, usually of the
///  form Machine<Option<T>>.
///
pub trait ChainMachine<T>: Machine<T> {
    type Result<X>;

    fn chain<U, M2>(self, next: M2) -> impl Machine<Self::Result<U>>
    where
        M2: Machine<Self::Result<U>, Transition = Self::Transition>;
}

pub trait JoinMachine<M1, T> {
    fn join(self) -> impl Machine<T>;
}

pub enum JoinedMachineState<M1, M2> {
    First(M1),
    Second(M2),
}

pub enum JoinedMachineIterator<I1, I2> {
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

pub struct JoinedMachine<T, M1, M2> {
    t: PhantomData<T>,
    state: JoinedMachineState<M1, M2>,
}

impl<T, M2, M1> Machine<Option<T>> for JoinedMachine<T, M1, M2>
where
    M1: Machine<Option<M2>, Transition = M2::Transition>,
    M2: Machine<Option<T>>,
{
    type Transition = M2::Transition;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        match &self.state {
            JoinedMachineState::First(i1) => JoinedMachineIterator::First(i1.edges()),
            JoinedMachineState::Second(i2) => JoinedMachineIterator::Second(i2.edges()),
        }
    }

    // fn state(&self) -> &Option<T> {
    //     match &self.state {
    //         JoinedMachineState::First(_) => &None,
    //         JoinedMachineState::Second(m2) => m2.state(),
    //     }
    // }

    fn state(&mut self) -> Option<T> {
        match &mut self.state {
            JoinedMachineState::First(_) => None,
            JoinedMachineState::Second(m2) => m2.state(),
        }
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        match &mut self.state {
            JoinedMachineState::First(m1) => m1.traverse(edge),
            JoinedMachineState::Second(m2) => m2.traverse(edge),
        };

        // Try to get the second machine from the first machine's state
        match &mut self.state {
            JoinedMachineState::First(m1) => {
                if let Some(m2) = m1.state() {
                    self.state = JoinedMachineState::Second(m2);
                }
            }
            _ => {}
        };
    }
}

pub struct ChainedMachine<T, M1, M2> {
    t: PhantomData<T>,
    in_second_machine: bool,
    machine1: M1,
    machine2: M2,
}

pub struct ChainedMachineIterator<I1, I2> {
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

impl<T, U, M1: Machine<Option<T>>, M2: Machine<Option<U>, Transition = M1::Transition>>
    Machine<Option<U>> for ChainedMachine<T, M1, M2>
{
    type Transition = M1::Transition;

    fn edges(&self) -> impl Iterator<Item = M1::Transition> {
        ChainedMachineIterator {
            in_second_machine: self.in_second_machine,
            iterator1: self.machine1.edges(),
            iterator2: self.machine2.edges(),
        }
    }

    fn state(&mut self) -> Option<U> {
        if self.in_second_machine {
            self.machine2.state()
        } else {
            None
        }
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        if self.in_second_machine {
            self.machine2.traverse(edge);
        } else {
            self.machine1.traverse(edge);
        }
    }
}

impl<T, M1: Machine<Option<T>>> ChainMachine<Option<T>> for M1 {
    type Result<X> = Option<X>;

    #[inline]
    fn chain<U, M2>(self, next: M2) -> impl Machine<Option<U>>
    where
        M2: Machine<Option<U>, Transition = M1::Transition>,
    {
        ChainedMachine {
            t: PhantomData,
            in_second_machine: false,
            machine1: self,
            machine2: next,
        }
    }
}

impl<T, M1: Machine<Option<T>>, M2: Machine<Option<M1>, Transition = M1::Transition>>
    JoinMachine<M1, Option<T>> for M2
{
    #[inline]
    fn join(self) -> impl Machine<Option<T>>
    where
        Self: Machine<Option<M1>>,
    {
        JoinedMachine {
            t: PhantomData,
            state: JoinedMachineState::First::<M2, M1>(self),
        }
    }
}

#[inline]
fn pure<T: Clone, E>(value: T) -> impl Machine<T> {
    PureMachine {
        e: PhantomData::<E>,
        value,
    }
}

pub struct PureMachine<T, E> {
    e: PhantomData<E>,
    value: T,
}

impl<T: Clone, E> Machine<T> for PureMachine<T, E> {
    type Transition = E;

    fn edges(&self) -> impl Iterator<Item = E> {
        empty()
    }

    fn state(&mut self) -> T {
        self.value.clone()
    }

    fn traverse(&mut self, _edge: &Self::Transition) {}
}

///
/// MappedMachine allows for mapping Machine transition type into another type while maintaining
/// the same semantics.
///
pub struct MappedMachine<T, M, F> {
    t: PhantomData<T>,
    machine: M,
    f: F,
}

impl<M, F, T, U: Clone> Machine<U> for MappedMachine<T, M, F>
where
    M: Machine<T>,
    F: Fn(&T) -> &U,
{
    type Transition = M::Transition;

    fn edges(&self) -> impl Iterator<Item = M::Transition> {
        self.machine.edges()
    }

    fn state(&mut self) -> U {
        (self.f)(&self.machine.state()).clone()
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        self.machine.traverse(edge);
    }
}
