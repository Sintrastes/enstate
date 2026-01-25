use core::marker::PhantomData;

///
/// Trait for objects that can act like a lens.
///
pub trait IsLens<A, S> {
    fn get(&self, source: S) -> A;

    fn set(&self, source: S, focus: A) -> S;
}

///
/// Trait for objects that can act like a prism.
///
pub trait IsPrism<A, S> {
    fn downcast(&self, source: S) -> Option<A>;

    fn upcast(&self, value: A) -> S;
}

///
/// Struct for a basic prism which implements the
///  IsPrism trait.
///
pub struct Prism<S, A, F: Fn(S) -> Option<A>, G: Fn(A) -> S> {
    s: PhantomData<S>,
    a: PhantomData<A>,
    downcast: F,
    upcast: G,
}

impl<S, A, F: Fn(S) -> Option<A>, G: Fn(A) -> S> Prism<S, A, F, G> {
    ///
    /// Build a new prism by specifying a downcast and upcast operation.
    ///
    pub fn new(downcast: F, upcast: G) -> impl IsPrism<A, S> {
        Prism {
            s: PhantomData,
            a: PhantomData,
            downcast,
            upcast,
        }
    }
}

impl<S, A, F: Fn(S) -> Option<A>, G: Fn(A) -> S> IsPrism<A, S> for Prism<S, A, F, G> {
    fn downcast(&self, source: S) -> Option<A> {
        (self.downcast)(source)
    }

    fn upcast(&self, value: A) -> S {
        (self.upcast)(value)
    }
}
