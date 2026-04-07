use enstate::machine::Machine;
use enstate::machine::proc_helpers::*;
use enstate_macros::proc;

// ---------------------------------------------------------------------------
// Linear pipeline
//
//   proc event -> do
//     let doubled = event * 2
//     let result  = doubled + 1
//     returnA -< result
//
// Uses the env-tuple path (let bindings always go through the general path).
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
    assert_eq!(*m.state(), 1); // (0 * 2) + 1

    m.traverse(-3);
    assert_eq!(*m.state(), -5); // (-3 * 2) + 1
}

// ---------------------------------------------------------------------------
// Non-linear pipeline
//
//   proc event -> do
//     let a   = event + 1    -- uses event
//     let b   = event * 2    -- also uses event (both stay in env)
//     let sum = a + b
//     returnA -< sum
//
// All variables remain in the env tuple, so any let binding can reference
// any previously bound variable including the original input.
// ---------------------------------------------------------------------------

#[test]
fn test_proc_non_linear_fanout() {
    let mut m = proc!(|event: i32| {
        let a = event + 1;
        let b = event * 2;
        let sum = a + b;
        returnA -< sum;
    });

    // event=5: a=6, b=10, sum=16
    m.traverse(5);
    assert_eq!(*m.state(), 16);

    // event=0: a=1, b=0, sum=1
    m.traverse(0);
    assert_eq!(*m.state(), 1);

    // event=-3: a=-2, b=-6, sum=-8
    m.traverse(-3);
    assert_eq!(*m.state(), -8);
}
