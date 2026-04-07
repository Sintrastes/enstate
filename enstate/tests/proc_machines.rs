//! Integration tests: `proc!` notation with `Machine` values.
//!
//! All combinators are imported from `enstate::machine::proc_helpers` so the
//! macro's generated bare calls to `arr`, `compose`, `fanout`, and `returnA`
//! resolve to the machine-oriented implementations.

#![feature(
    coroutines,
    coroutine_trait,
    trait_alias,
    never_type,
    exhaustive_patterns
)]

use enstate::machine::Machine;
use enstate::machine::accumulator::accumulator;
use enstate::machine::proc_helpers::{arr, compose, cycle, fanout};
use enstate_macros::proc;

// ---------------------------------------------------------------------------
// Linear pipeline
// ---------------------------------------------------------------------------

#[test]
fn test_proc_linear_pipeline() {
    let mut m = proc!(|event: i32| {
        let doubled = event * 2;
        let result = doubled + 1;
        returnA -< result;
    });

    m.traverse(5);
    assert_eq!(*m.state(), 11); // (5 * 2) + 1

    m.traverse(0);
    assert_eq!(*m.state(), 1);

    m.traverse(-3);
    assert_eq!(*m.state(), -5);
}

// ---------------------------------------------------------------------------
// Non-linear pipeline
// ---------------------------------------------------------------------------

#[test]
fn test_proc_non_linear_fanout() {
    let mut m = proc!(|event: i32| {
        let a = event + 1;
        let b = event * 2;
        let sum = a + b;
        returnA -< sum;
    });

    m.traverse(5);
    assert_eq!(*m.state(), 16); // 6 + 10

    m.traverse(0);
    assert_eq!(*m.state(), 1);

    m.traverse(-3);
    assert_eq!(*m.state(), -8);
}

// ---------------------------------------------------------------------------
// Linear pipeline with a stateful machine
// ---------------------------------------------------------------------------

#[test]
fn test_proc_stateful_linear() {
    let mut m = proc!(|event: i32| {
        running_sum <- accumulator(0i32, |acc: &i32, e: &i32| acc + e) -< event;
        let scaled = running_sum * 10;
        returnA -< scaled;
    });

    m.traverse(3);
    assert_eq!(*m.state(), 30);

    m.traverse(4);
    assert_eq!(*m.state(), 70);

    m.traverse(-2);
    assert_eq!(*m.state(), 50);
}

// ---------------------------------------------------------------------------
// Non-linear pipeline with two stateful machines
// ---------------------------------------------------------------------------

#[test]
fn test_proc_stateful_non_linear() {
    let mut m = proc!(|event: i32| {
        count <- accumulator(0i32, |acc: &i32, _: &i32| acc + 1) -< event;
        total <- accumulator(0i32, |acc: &i32, e: &i32| acc + e) -< event;
        let combo = count * total;
        returnA -< combo;
    });

    m.traverse(5);
    assert_eq!(*m.state(), 5); // count=1, total=5

    m.traverse(3);
    assert_eq!(*m.state(), 16); // count=2, total=8

    m.traverse(-2);
    assert_eq!(*m.state(), 18); // count=3, total=6
}

// ---------------------------------------------------------------------------
// let inside rec: pure computation alongside stateful feedback
// ---------------------------------------------------------------------------

#[test]
fn test_proc_rec_let() {
    // let inside rec: `scaled` is a pure expression of the feedback var `total`
    let mut m = proc!(|event: i32| {
        rec(0i32) {
            total  <- arr(|(e, acc): (i32, i32)| e + acc) -< (event, total);
            let scaled = total * 2;
        };
        returnA -< scaled;
    });

    m.traverse(3);
    assert_eq!(*m.state(), 6); // total=3, scaled=6

    m.traverse(4);
    assert_eq!(*m.state(), 14); // total=7, scaled=14

    m.traverse(-2);
    assert_eq!(*m.state(), 10); // total=5, scaled=10
}

// ---------------------------------------------------------------------------
// Field access via input type annotation
// ---------------------------------------------------------------------------

#[test]
fn test_proc_field_access() {
    #[derive(Clone)]
    struct Event {
        value: i32,
        scale: i32,
    }

    let mut m = proc!(|event: Event| {
        let result = event.value * event.scale;
        returnA -< result;
    });

    m.traverse(Event { value: 3, scale: 4 });
    assert_eq!(*m.state(), 12);

    m.traverse(Event { value: -2, scale: 5 });
    assert_eq!(*m.state(), -10);
}

// ---------------------------------------------------------------------------
// Field access on a Bind-output variable in a non-leading let
// ---------------------------------------------------------------------------

#[test]
fn test_proc_let_field_on_bind_output() {
    #[derive(Clone)]
    struct Vel { value: i32 }

    let mut m = proc!(|event: i32| {
        vel: Vel <- accumulator(Vel { value: 0 }, |acc: &Vel, e: &i32| Vel { value: acc.value + e }) -< event;
        let doubled = vel.value * 2;
        returnA -< doubled;
    });

    m.traverse(3);
    assert_eq!(*m.state(), 6);  // vel=3, doubled=6

    m.traverse(4);
    assert_eq!(*m.state(), 14); // vel=7, doubled=14
}

// ---------------------------------------------------------------------------
// Field access on input var in a non-leading let (after a Bind)
// ---------------------------------------------------------------------------

#[test]
fn test_proc_let_field_on_input_after_bind() {
    #[derive(Clone)]
    struct Event { value: i32 }

    let mut m = proc!(|event: Event| {
        count <- accumulator(0i32, |acc: &i32, _: &Event| acc + 1) -< event;
        let result = event.value * count;
        returnA -< result;
    });

    m.traverse(Event { value: 3 });
    assert_eq!(*m.state(), 3);  // value=3, count=1

    m.traverse(Event { value: 5 });
    assert_eq!(*m.state(), 10); // value=5, count=2
}

// ---------------------------------------------------------------------------
// rec block: self-referential running total
//
//   proc event -> do
//     rec(0i32) {
//       total <- arr(|(e, acc)| e + acc) -< (event, total);
//     };
//     returnA -< total
//
// `total` appears in its own input expression → it is the feedback var.
// The macro automatically wires cycle(0i32, ...) to feed back `total`.
// ---------------------------------------------------------------------------

#[test]
fn test_proc_rec_running_total() {
    let mut m = proc!(|event: i32| {
        rec(0i32) {
            total <- arr(|(e, acc): (i32, i32)| e + acc) -< (event, total);
        };
        returnA -< total;
    });

    m.traverse(3);
    assert_eq!(*m.state(), 3); // 0 + 3

    m.traverse(4);
    assert_eq!(*m.state(), 7); // 3 + 4

    m.traverse(-2);
    assert_eq!(*m.state(), 5); // 7 + (-2)
}

// ---------------------------------------------------------------------------
// rec block: mutual recursion (x uses previous y, y uses current x)
//
//   rec(0i32) {
//     x <- arr(|(i, y)| i + y) -< (event, y);  -- y is forward-ref (feedback)
//     y <- arr(|x| x * 2)      -< x;            -- x is same-step (not feedback)
//   };
//
// After each traversal:
//   x_new = event + y_prev
//   y_new = x_new * 2
// ---------------------------------------------------------------------------

#[test]
fn test_proc_rec_mutual() {
    let mut m = proc!(|event: i32| {
        rec(0i32) {
            x <- arr(|(i, y): (i32, i32)| i + y) -< (event, y);
            y <- arr(|x: i32| x * 2)             -< x;
        };
        returnA -< x;
    });

    // step 1: x = 5 + 0 = 5,  y = 10
    m.traverse(5);
    assert_eq!(*m.state(), 5);

    // step 2: x = 3 + 10 = 13, y = 26
    m.traverse(3);
    assert_eq!(*m.state(), 13);

    // step 3: x = -2 + 26 = 24, y = 48
    m.traverse(-2);
    assert_eq!(*m.state(), 24);
}

// ---------------------------------------------------------------------------
// rec block: Default::default() as initial feedback (no explicit init)
// ---------------------------------------------------------------------------

#[test]
fn test_proc_rec_default_init() {
    let mut m = proc!(|event: i32| {
        rec {
            total <- arr(|(e, acc): (i32, i32)| e + acc) -< (event, total);
        };
        returnA -< total;
    });

    m.traverse(10);
    assert_eq!(*m.state(), 10); // 0 + 10

    m.traverse(5);
    assert_eq!(*m.state(), 15); // 10 + 5
}

// ---------------------------------------------------------------------------
// Generic type annotation: `var: Type<T> <- ...`
// ---------------------------------------------------------------------------

#[test]
fn test_proc_generic_type_annotation() {
    #[derive(Clone)]
    struct Wrapper<T> { inner: T }

    let mut m = proc!(|event: i32| {
        w: Wrapper<i32> <- accumulator(
            Wrapper { inner: 0i32 },
            |acc: &Wrapper<i32>, e: &i32| Wrapper { inner: acc.inner + e },
        ) -< event;
        let result = w.inner * 3;
        returnA -< result;
    });

    m.traverse(4);
    assert_eq!(*m.state(), 12); // inner=4, result=12

    m.traverse(6);
    assert_eq!(*m.state(), 30); // inner=10, result=30
}
