# What is it?

In short: Enstate is a `#![no_std]` compatible Rust library for composable and flexible zero-cost
 state machine abstractions. Its design is inspired by that of `std::Iterator`, as well as previous work
 I've done on how to represent composable state machines in a functional way.

This all ultimately stems from ideas from Phil Friedman such as [Comonads As Spaces](https://blog.functorial.com/posts/2016-08-07-Comonads-As-Spaces.html)
 and [Declarative UIs are the Future — And the Future is Comonadic](https://functorial.com/the-future-is-comonadic/main.pdf) -- but made practical in a
 mainstream programming language, and extended to a new type of UI abstraction (UI workflows) which I believe is roughly dual
 (though I have yet to be able to exactly formulate this idea categorically) to Phil's work with composable UI widgets.

Whatever we want to call this "dual", I think that this aspect of UI development is often overlooked in modern
 declarative UI frameworks -- leading to awkward APIs for doing something that in a declarative framework _should_
 be simple, like launching a modal dialog (looking at you, Jetpack Compose!), so one of the goals of this library
 is to promote the idea of a "Composable UI Workflow" abstraction.

# How does it work?

A `Machine<T>` is a trait for a mutable state machine which at any time can be inspected for it's
 "current state" of type `T`. `Machine`s have an associated type of `Transition`s to represent the type
 of state transitions that a machine can undergo. This is usually some kind of `enum`. At any state, a
 `Machine` can transition to a subset of the set of possible `Transition`s, designated by `fn edges(&self)`,
 which returns an iterator of allowable transitions at the current state.

To ensure compose two machines, you'll usually need to ensure that their type of `Transition`s
 are the same. Thus, usually a good strategy for building machines that you can compose with
 each other is building a "big" `Transition` type representing all possible transitions in your
 application, and using the `fn edges(&self)` method of machines to pick out the subset of
 actually allowable transitions for a particular machine.

## Vertical Composition

Regular `Machine`s should be viewed as open-ended state machines without "end" states. They are designed
 to represent open-ended scenarios, like the state machine for an entire application, or for a UI widget.
 These may be composed "vertically" using `zipWith`, which merges the possible transitions at each state,
 and combines the underlying states with a supplied function.

To understand this better, let's look at an example:

```rust
struct Counter {
    count: i32
}

enum CounterAction {
    Increment,
    Decrement
}

impl Machine<i32> for Counter {
    type Transition = CounterAction;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        use CounterAction::*;
        std::iter::once(Increment)
            .chain(std::iter::once(Decrement))
    }

    fn state(&mut self) -> i32 {
        self.count
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        match edge {
            CounterAction::Increment => self.count += 1,
            CounterAction::Decrement => self.count -= 1,
        }
    }
}
```

This defines the state machine for a basic "counter" widget, an integer "count" displayed,
 with "actions" for incrementing and decrementing that count. Using the `machine!` macro from `enstate-macros`,
 and using some nightly rust features, we can actually much more simply define this as:

```rust
fn counter() -> impl StateMachine<CounterAction, i32> {
    machine!(count, 0, || {
        let action = choose![CounterAction::Increment, CounterAction::Decrement];
        match action {
            CounterAction::Increment => count = count + 1,
            CounterAction::Decrement => count = count - 1,
        }
    })
}
```

What if we wanted to build a state machine for two counters, where we sum the two states? `zipWith`
takes two machines, and builds a new machine that allows us to traverse along either the first machine's
or the second machine's transitions, building up a joint state by combining the individual states with
a function.

```rust
enum AdderActions {
    Counter1(CounterAction),
    Counter2(CounterAction)
}

let counter1 = Counter { count: 0 }
    .lift(|event| Counter1(event));

let counter2 = Counter { count: 1 }
    .lift(|event| Counter2(event));

let adder = counter1
    .zipWith(counter2, |count1, count2| count1 + count2 );
```

For those familiar with abstractions from functional programming, this essentially makes
 `Machine` an `Applicative`.

## Horizontal Composition

The interesting thing about machines is that for particular `Machine<T>`s, they also admit
 a different kind of composition that we call here a "horizontal" composition -- to evoke an image of
 composing machines _in sequence_.

For machines admitting this type of composition, we use the traits `ChainMachine` and `JoinMachine`.
 `Machine<Option<T>>` is an instance of both for any `T`.

The idea behind this abstraction is that each state of the machine can be classified into "final"
 and "non-final" states, which gives us junctions by which we can "glue" one machine to the
 "end" of the other. For example, for the `Option` instance, `None` states are viewed as "non-final",
 with `Some(...)` values represnting the "final" return values of a machine.

For example, rather than a UI widget like we've considered earlier (which has an indeterminate lifecycle),
 let's consider the state space of something with a determinate start and end -- a modal dialog.

```rust
enum ModalResult<T> {
    Ok(T),
    Cancelled
}

struct CounterModalState {
    result: Option<ModalResult<i32>>
}

impl Machine<Option<ModalResult<i32>>> for ModalState {
    ...
}
```
