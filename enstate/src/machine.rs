use core::{cell::RefCell, marker::PhantomData};

use mapped::{MappedMachine, MappedTransitionMachine};
use zipped::ZippedMachine;

use crate::{
    machine::{pipe::PipedMachine, post_map::PostMap, pre_map::PreMap},
    optics::IsPrism,
};

pub mod accumulator;
pub mod accumulator_mut;
pub mod arr;
pub mod chained;
pub mod constant;
pub mod integral;
pub mod mapped;
mod pipe;
mod post_map;
mod pre_map;
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
    /// Get a reference to the current state of the machine.
    ///
    fn state(&self) -> &T;

    ///
    /// Traverse along an edge to update the state of the
    ///  machine.
    ///
    /// If the transition is not in the current edges,
    ///  this should be a no-op.
    ///
    fn traverse(&mut self, edge: Self::Transition);

    ///
    /// Traverse along an edge by reference to update the state of the
    ///  machine.
    ///
    /// This is useful when the same edge needs to be used for multiple
    ///  machines, avoiding clones.
    ///
    /// If the transition is not in the current edges,
    ///  this should be a no-op.
    ///
    fn traverse_ref(&mut self, edge: &Self::Transition)
    where
        Self::Transition: Clone,
    {
        self.traverse(edge.clone())
    }

    ///
    /// Transform the state of a machine by applying a function.
    ///
    #[inline]
    fn map<F, U>(self, f: F) -> impl Machine<U, Transition = Self::Transition>
    where
        F: Fn(&T) -> U,
    {
        MappedMachine {
            u: PhantomData,
            t: PhantomData,
            machine: self,
            f,
            cached_state: RefCell::new(None),
        }
    }

    ///
    /// Transform a machine by embedding the events into a larger
    ///  type of events.
    ///
    #[inline]
    fn lift<E, P: IsPrism<Self::Transition, E>>(
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
    fn zip_with<M2, U, G, W>(
        self,
        machine2: M2,
        f: G,
    ) -> impl Machine<W, Transition = Self::Transition>
    where
        M2: Machine<U, Transition = Self::Transition>,
        Self::Transition: Clone,
        G: Fn(&T, &U) -> W,
        W: Clone,
    {
        ZippedMachine {
            t: PhantomData,
            u: PhantomData,
            machine1: self,
            machine2,
            f,
            cached_state: RefCell::new(None),
        }
    }

    /// Adapt the state type of a machine by applying an operation to the output type.
    fn rmap<U, F: Fn(&T) -> U>(self, f: F) -> impl Machine<U, Transition = Self::Transition> {
        PostMap::new(self, f)
    }

    ///Adapt the transition type of a mchine by pre-applying a function to its transition type.
    fn lmap<U, F: Fn(&U) -> Self::Transition>(self, f: F) -> impl Machine<T, Transition = U> {
        PreMap::new(self, f)
    }

    /// Compose two machines by piping the output of one into the input of another.
    fn pipe<U, M: Machine<U, Transition = T>>(
        self, // Machine<T, Transition = Self::Transition>
        other: M,
    ) -> impl Machine<U, Transition = Self::Transition>
    where
        Self::Transition: Clone,
        T: Clone,
    {
        PipedMachine {
            t: PhantomData,
            u: PhantomData,
            machine1: self,
            machine2: other,
        }
    }
}
