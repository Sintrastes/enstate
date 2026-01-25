use core::marker::PhantomData;

use mapped::{MappedMachine, MappedTransitionMachine};
use zipped::ZippedMachine;

use crate::optics::{IsPrism, Prism};

pub mod chained;
pub mod mapped;
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
    /// Transform a machine by embedding the events into a larger
    ///  type of events.
    ///
    #[inline]
    fn lift<E: Clone, P: IsPrism<Self::Transition, E>>(
        self,
        prism: &P,
    ) -> impl Machine<T, Transition = E> {
        MappedTransitionMachine {
            t: PhantomData,
            machine: self,
            f: |x| prism.upcast(x),
            g: |x| prism.downcast(x),
        }
    }

    ///
    /// Combine two machines "horizontally", combinding their state with a function.
    ///
    #[inline]
    fn zip_with<M2, U, G, W: Clone>(
        self,
        machine2: M2,
        f: G,
    ) -> impl Machine<W, Transition = Self::Transition>
    where
        M2: Machine<U, Transition = Self::Transition>,
        // M2::Transition: PartialEq<Self::Transition>,
        G: FnMut(T, U) -> W,
    {
        ZippedMachine {
            t: PhantomData,
            u: PhantomData,
            machine1: self,
            machine2,
            f,
        }
    }
}
