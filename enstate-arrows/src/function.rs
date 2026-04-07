//! Arrow implementation for pure functions.

pub fn arr<A, B, F>(f: F) -> F
where
    F: Fn(A) -> B,
{
    f
}

pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

pub fn fanout<A: Clone, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> (B, C)
where
    F: Fn(A) -> B,
    G: Fn(A) -> C,
{
    move |x| (f(x.clone()), g(x.clone()))
}

pub fn returnA<A>() -> impl Fn(A) -> A {
    |x| x
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use crate::function::{arr, compose, fanout, returnA};
    use enstate_macros::proc;

    // Non-linear: each bind reaches back to variables earlier than the
    // immediately preceding one, so the general fanout path is used.
    #[test]
    fn test_proc_non_linear() {
        fn sum_of_pair() -> impl Fn(i32) -> i32 {
            proc!(|x| {
                y    <- arr(|x: i32| x + 1)              -< x;
                z    <- arr(|x: i32| x * 2)              -< x;
                pair <- arr(|x: (i32, i32)| x)           -< (y, z);
                sum  <- arr(|(a, b): (i32, i32)| a + b)  -< pair;
                returnA -< sum;
            })
        }

        let f = sum_of_pair();
        assert_eq!(f(5), 6 + 10);
        assert_eq!(f(0), 1 + 0);
        assert_eq!(f(-3), -2 + -6);
    }

    // Linear (arrowp "trivial" case):
    //   proc inputs -> do
    //     chunked <- chunk  -< inputs
    //     results <- process -< chunked
    //     returnA -< results
    // must desugar to  compose(chunk, process)  with no fanout or env tuples.
    // We verify by checking that proc! and direct compose give identical results.
    #[test]
    fn test_proc_linear_matches_compose() {
        let chunk = arr(|x: i32| x * 3);
        let process = arr(|x: i32| x - 1);

        let via_proc: &dyn Fn(i32) -> i32 = &proc!(|inputs| {
            chunked <- arr(|x: i32| x * 3) -< inputs;
            results <- arr(|x: i32| x - 1) -< chunked;
            returnA -< results;
        });

        let direct: &dyn Fn(i32) -> i32 = &compose(chunk, process);

        for i in -5..=5 {
            assert_eq!(via_proc(i), direct(i), "mismatch at input {i}");
        }
    }

    // Linear pipeline with three steps.
    #[test]
    fn test_proc_linear_three_steps() {
        fn pipeline() -> impl Fn(i32) -> i32 {
            proc!(|x| {
                a <- arr(|x: i32| x * 2)  -< x;
                b <- arr(|x: i32| x + 10) -< a;
                c <- arr(|x: i32| x * x)  -< b;
                returnA -< c;
            })
        }
        // Expected: ((x*2)+10)^2
        let f = pipeline();
        assert_eq!(f(3), ((3 * 2) + 10) * ((3 * 2) + 10)); // 16^2 = 256
        assert_eq!(f(0), ((0 * 2) + 10) * ((0 * 2) + 10)); // 10^2 = 100
        assert_eq!(f(-5), ((-5 * 2) + 10) * ((-5 * 2) + 10)); // 0^2  = 0
    }

    // Single-step linear pipeline: proc |x| { y <- f -< x; returnA -< y; }
    // should desugar to just f.
    #[test]
    fn test_proc_linear_single_step() {
        let f = proc!(|x| {
            y <- arr(|x: i32| x + 42) -< x;
            returnA -< y;
        });
        assert_eq!(f(0), 42);
        assert_eq!(f(8), 50);
    }
}
