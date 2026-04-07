use core::iter::empty;
use core::marker::PhantomData;

#[cfg(feature = "std")]
use std::sync::Arc;

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
pub mod cycle;
pub mod delay;
pub mod event;
pub mod fanout;
pub mod integral;
pub mod mapped;
pub mod passthrough;
mod pipe;
mod post_map;
mod pre_map;
pub mod proc_helpers;
pub mod zipped;

///
/// Trait for a composable state machine with state of type T.
///
/// State machines have an associated type Transition reprenting
///  the "edges" that can be traversed in a state transition.
///
/// Generally speaking only machines with the same transition type can be composed.
///
pub trait Machine<I: Send, O: Send + Sync>: Send + Sync {
    // Idea: "edges" could be made more efficent if it returned an enum
    //  either Edges::Unconstrainted, representing the fact that any transition
    //  is a valid transition, or Edges::Constrainted(...), together with some
    //  sort of representation of an edge constraint (such as a slice, iterator,
    //  or perhaps even something else).
    //
    // But something difficult with this is trying to get this to work with "dynamic edges"
    //  like in the Haskell prototype in a way that would be useful in guiding testing
    //  frameworks.
    //
    // Maybe we could come up with a trait for transition types and come up wtih
    //  a convinient derive macro for it.
    //

    ///
    /// Get the possible edges which can be used to transition
    /// out of the current state of the machine.
    ///
    fn edges(&self) -> impl Iterator<Item = I>
    where
        Self: Sized;

    ///
    /// Get a reference to the current state of the machine.
    ///
    fn state(&mut self) -> &O;

    ///
    /// Traverse along an edge to update the state of the
    ///  machine.
    ///
    /// If the transition is not in the current edges,
    ///  this should be a no-op.
    ///
    fn traverse(&mut self, edge: I);

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
    fn traverse_ref(&mut self, edge: &I);

    ///
    /// Transform the state of a machine by applying a function.
    ///
    #[inline]
    fn map<F, U: Send + Sync + Clone>(self, f: F) -> impl Machine<I, U> + Clone + Sync
    where
        Self: Sized + Sync + Clone,
        I: Send + Sync + Clone,
        O: Send + Clone,
        F: Fn(&O) -> U + Send + Sync + Clone,
    {
        MappedMachine {
            u: PhantomData,
            t: PhantomData,
            machine: self,
            f,
            cached_state: None,
        }
    }

    ///
    /// Transform a machine by embedding the events into a larger
    ///  type of events.
    ///
    #[inline]
    fn lift<E: Send, P: IsPrism<I, E> + Sync>(self, prism: &P) -> impl Machine<E, O> + Clone
    where
        I: Send,
        O: Send + Clone,
        Self: Sized + Clone,
    {
        MappedTransitionMachine {
            t: PhantomData,
            machine: self,
            f: |x: &I| prism.upcast(x),
            g: |x: &E| prism.downcast(&x),
        }
    }

    ///
    /// Combine two machines "horizontally", combinding their state with a function.
    ///
    #[inline]
    fn zip_with<M2, U: Send + Clone + Sync, G, W: Send + Sync + Clone>(
        self,
        machine2: M2,
        f: G,
    ) -> impl Machine<I, W> + Clone
    where
        Self: Sized + Clone,
        M2: Machine<I, U> + Clone,
        I: Clone + Send + Sync,
        O: Clone,
        G: Fn(&O, &U) -> W + Send + Sync + Clone,
    {
        ZippedMachine {
            t: PhantomData,
            u: PhantomData,
            machine1: self,
            machine2,
            f,
            cached_state: None,
        }
    }

    ///
    /// Combine two machines "horizontally", combinding their state with a function.
    ///
    #[inline]
    fn zip<M2, U>(self, machine2: M2) -> impl Machine<I, (O, U)> + Clone
    where
        Self: Sized + Clone,
        M2: Machine<I, U> + Clone,
        I: Clone + Send + Sync,
        O: Clone + Send,
        U: Clone + Send + Sync,
    {
        self.zip_with(machine2, |x, y| (x.clone(), y.clone()))
    }

    /// Adapt the state type of a machine by applying an operation to the output type.
    fn rmap<U: Send + Sync + Clone, F: Fn(&O) -> U + Send + Sync + Clone>(
        self,
        f: F,
    ) -> impl Machine<I, U> + Clone + Clone
    where
        I: Send + Sync,
        O: Send + Clone,
        Self: Sized + Clone,
    {
        PostMap::new(self, f)
    }

    ///Adapt the transition type of a mchine by pre-applying a function to its transition type.
    fn lmap<U: Send + Sync + Clone, F: Fn(&U) -> I + Send + Sync + Clone>(
        self,
        f: F,
    ) -> impl Machine<U, O> + Clone
    where
        Self: Sized + Clone,
        O: Clone,
    {
        PreMap::new(self, f)
    }

    /// Compose two machines by piping the output of one into the input of another.
    fn pipe<U: Send + Sync + Clone, M: Machine<O, U> + Clone>(
        self, // Machine<T, Transition = Self::Transition>
        other: M,
    ) -> impl Machine<I, U> + Clone
    where
        Self: Sized + Clone,
        I: Clone,
        O: Clone,
    {
        PipedMachine {
            t: PhantomData,
            u: PhantomData,
            machine1: self,
            machine2: other,
        }
    }

    ///
    /// Constructs a machine that references a shared state, updating the value once
    ///  per frame.
    ///
    #[cfg(feature = "std")]
    fn by_arc(reference: Arc<O>) -> impl Machine<I, O> + Clone
    where
        Self: Sized + Clone,
        I: Clone + Send + Sync,
        O: Clone + Send + Sync,
    {
        RefMachine {
            t: PhantomData,
            u: PhantomData,
            reference: reference.clone(),
            cached_value: None,
        }
    }
}

#[cfg(feature = "std")]
#[derive(Clone)]
struct RefMachine<I: Clone, O: Clone> {
    t: PhantomData<I>,
    u: PhantomData<O>,
    reference: Arc<O>,
    cached_value: Option<O>,
}

#[cfg(feature = "std")]
impl<I: Clone + Send + Sync, O: Clone + Send + Sync> Machine<I, O> for RefMachine<I, O> {
    fn edges(&self) -> impl Iterator<Item = I>
    where
        Self: Sized,
    {
        empty()
    }

    fn state(&mut self) -> &O {
        &self.cached_value.as_ref().unwrap()
    }

    fn traverse(&mut self, edge: I) {
        // On traversal, update the state to the latest value.
        self.cached_value = Some(self.reference.as_ref().clone());
    }

    fn traverse_ref(&mut self, edge: &I) {
        // On traversal, update the state to the latest value.
        self.cached_value = Some(self.reference.as_ref().clone());
    }
}
