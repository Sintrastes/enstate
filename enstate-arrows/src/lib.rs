#![no_std]
#![feature(impl_trait_in_assoc_type)]

//! Arrow abstraction for composable computations.
//!
//! This crate provides combinators for building arrows out of plain `Fn` values,
//! along with the [`ArrowOps`] and [`ArrowChoice`] traits for defining arrow abstractions.
//!
//! # Example
//!
//! ```
//! use enstate_arrows::function::{arr, compose, fanout};
//!
//! let double = arr(|x: i32| x * 2);
//! let add_one = arr(|x: i32| x + 1);
//! let composed = compose(double, add_one);
//!
//! // (5 * 2) + 1 = 11
//! assert_eq!(composed(5), 11);
//!
//! // fanout runs both arrows on the same input
//! let both = fanout(arr(|x: i32| x * 2), arr(|x: i32| x + 1));
//! assert_eq!(both(3), (6, 4));
//! ```

/// Core Arrow operations trait.
///
/// This trait defines the fundamental operations that any arrow must support:
/// - `pipe`: Sequential composition of arrows
/// - `first`: Apply arrow to first component of a tuple
///
/// # Type Parameters
/// - `I`: Input type
/// - `O`: Output type
pub trait ArrowOps<I, O>: Sized + ApplyArrow<I, O> {
    /// Compose this arrow with another arrow sequentially.
    ///
    /// The output of this arrow becomes the input to the next arrow.
    fn pipe<O2, A2>(self, next: A2) -> impl ArrowOps<I, O2>
    where
        O: Clone,
        O2: Clone,
        A2: ArrowOps<O, O2> + ApplyArrow<O, O2>;

    /// Apply this arrow to the first component of a tuple, leaving the second unchanged.
    ///
    /// This is fundamental for routing data through arrow computations.
    fn first<C: Clone>(self) -> impl ArrowOps<(I, C), (O, C)>
    where
        O: Clone;
}

/// Extended arrow operations for choice and parallel composition.
pub trait ArrowChoice<I, O>: ArrowOps<I, O> {
    /// Parallel composition: run both arrows on the same input and combine outputs.
    ///
    /// Also known as "fanout" or the `&&&` operator in Haskell.
    fn fanout<O2>(self, other: impl ArrowOps<I, O2>) -> impl ArrowOps<I, (O, O2)>
    where
        I: Clone,
        O: Clone,
        O2: Clone;
}

/// Helper trait for applying arrows.
///
/// This is used internally to allow applying arrows without knowing
/// the exact concrete type.
pub trait ApplyArrow<I, O> {
    fn apply_arrow(&self, input: &I) -> O;
}

pub mod function;
