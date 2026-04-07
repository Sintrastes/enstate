//! Free functions for use with the [`proc!`] macro on [`Machine`] values.
//!
//! The `proc!` macro desugars to bare calls to `arr`, `compose`, `fanout`,
//! and `returnA` in the invoking scope.  This module provides the four
//! functions that match the calling convention the macro expects:
//!
//! | function  | what it wraps                                  |
//! |-----------|------------------------------------------------|
//! | `arr`     | [`machine::arr::arr`] — adapts owned → `&I`   |
//! | `compose` | [`Machine::pipe`] — standalone free function  |
//! | `fanout`  | [`machine::fanout::FanoutMachine`] — with `+ Clone` |
//! | `returnA` | identity machine built on `arr`                |
//!
//! # Example
//!
//! ```
//! use enstate::machine::Machine;
//! use enstate::machine::proc_helpers::{arr, compose, fanout, returnA};
//! use enstate_macros::proc;
//!
//! let mut m = proc!(|event| {
//!     doubled <- arr(|x: i32| x * 2) -< event;
//!     result  <- arr(|x: i32| x + 1) -< doubled;
//!     returnA -< result;
//! });
//!
//! m.traverse(5);
//! assert_eq!(*m.state(), 11); // (5 * 2) + 1
//! ```

use super::Machine;

/// Lifts a pure function into a [`Machine`].
///
/// Unlike the low-level [`machine::arr::arr`], this version accepts
/// `Fn(I) -> O` (owned input), matching the calling convention assumed
/// by the `proc!` macro. The input is cloned from the `&I` reference
/// that machines receive internally.
pub fn arr<I, O, F>(f: F) -> impl Machine<I, O> + Clone
where
    I: Clone + Send + Sync,
    O: Clone + Send + Sync,
    F: Fn(I) -> O + Clone + Send + Sync,
{
    super::arr::arr(move |x: &I| f(x.clone()))
}

/// Composes two machines sequentially — equivalent to `m1.pipe(m2)`.
///
/// The state of `m1` after each traversal is forwarded as the input to `m2`.
pub fn compose<I, T, O, M1, M2>(m1: M1, m2: M2) -> impl Machine<I, O> + Clone
where
    M1: Machine<I, T> + Clone,
    M2: Machine<T, O> + Clone,
    I: Clone + Send + Sync,
    T: Clone + Send + Sync,
    O: Clone + Send + Sync,
{
    m1.pipe(m2)
}

/// Runs two machines on the same input and pairs their states.
///
/// Wraps [`machine::fanout::FanoutMachine`] directly so the return type
/// carries `+ Clone`, which `compose` requires of its arguments.
pub fn fanout<I, A, B, M1, M2>(m1: M1, m2: M2) -> impl Machine<I, (A, B)> + Clone
where
    I: Clone + Send + Sync,
    A: Clone + Send + Sync,
    B: Clone + Send + Sync,
    M1: Machine<I, A> + Clone,
    M2: Machine<I, B> + Clone,
{
    super::fanout::FanoutMachine::new(m1, m2)
}

/// Identity machine — emits its input as its state unchanged.
pub fn returnA<I: Clone + Send + Sync>() -> impl Machine<I, I> + Clone {
    arr(|x: I| x)
}

/// Loops a machine by feeding the second component of its output back as the
/// second component of its next input.
///
/// `initial` is the feedback value used on the very first traversal.
pub fn cycle<I, O, L, M>(initial: L, machine: M) -> impl Machine<I, O> + Clone
where
    M: Machine<(I, L), (O, L)> + Clone,
    I: Clone + Send + Sync,
    O: Clone + Send + Sync,
    L: Clone + Send + Sync,
{
    super::cycle::cycle(initial, machine)
}
