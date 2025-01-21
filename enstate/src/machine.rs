use core::iter::empty;

use core::marker::PhantomData;

use mapped::{MappedMachine, MappedTransitionMachine};
use zipped::ZippedMachine;

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
    /// Transform the state of a machine by applying a function.
    ///
    #[inline]
    fn map_actions<F, G, E>(self, f: F, g: G) -> MappedTransitionMachine<T, Self, F, G>
    where
        F: Fn(Self::Transition) -> E,
        G: Fn(E) -> Option<Self::Transition>,
    {
        MappedTransitionMachine {
            t: PhantomData,
            machine: self,
            f,
            g,
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
    ) -> impl Machine<W, Transition = E>
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
            t: PhantomData,
            u: PhantomData,
            machine1: self.map_actions(|x| x.into(), |x| x.try_into().ok()),
            machine2: machine2.map_actions(|x| x.into(), |x| x.try_into().ok()),
            f,
        }
    }

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
