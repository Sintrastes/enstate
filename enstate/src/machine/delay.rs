//! Delay combinator for holding previous values.
//!
//! This module provides the `Delay` state machine, which outputs the
//! previous input value. This is analogous to Yampa's `iPre` (initial
//! previous) combinator.
//!
//! # Example
//! ```
//! use enstate::machine::Machine;
//! use enstate::machine::delay::delay;
//!
//! // Create a delay that holds the previous value
//! let mut delayed = delay(0);
//!
//! // Initially outputs the initial value
//! assert_eq!(*delayed.state(), 0);
//!
//! delayed.traverse(10);
//! assert_eq!(*delayed.state(), 0); // Still outputs initial
//!
//! delayed.traverse(20);
//! assert_eq!(*delayed.state(), 10); // Now outputs previous input
//! ```

use crate::machine::Machine;

/// A state machine with a delay (holds previous value).
///
/// This machine outputs the value from the previous time step, which is
/// useful for detecting changes, creating feedback loops, or implementing
/// one-step memory in signal processing.
///
/// # Type Parameters
/// - `T` - The type of values being delayed
#[derive(Clone)]
pub struct Delay<T: Copy> {
    current: T,
    previous: T,
}

impl<T: Copy + Send + Sync> Machine<T, T> for Delay<T> {
    fn edges(&self) -> impl Iterator<Item = T> {
        // Delay machines accept any input
        core::iter::empty()
    }

    fn state(&mut self) -> &T {
        &self.previous
    }

    fn traverse(&mut self, event: T) {
        self.previous = self.current;
        self.current = event;
    }

    fn traverse_ref(&mut self, event: &T) {
        self.previous = self.current;
        self.current = *event;
    }
}

/// Creates a delay machine that outputs the previous input value.
///
/// # Arguments
/// - `initial` - The initial value to use before any input is received
///
pub fn delay<T: Copy>(initial: T) -> Delay<T> {
    Delay {
        current: initial,
        previous: initial,
    }
}
