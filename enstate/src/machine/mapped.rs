use core::marker::PhantomData;

use super::Machine;

///
/// MappedMachine allows for mapping Machine transition type into another type while maintaining
/// the same semantics.
///
pub struct MappedMachine<T, M, F> {
    pub t: PhantomData<T>,
    pub machine: M,
    pub f: F,
}

impl<M, F, T, U: Clone> Machine<U> for MappedMachine<T, M, F>
where
    M: Machine<T>,
    F: Fn(&T) -> &U,
{
    type Transition = M::Transition;

    fn edges(&self) -> impl Iterator<Item = M::Transition> {
        self.machine.edges()
    }

    fn state(&mut self) -> U {
        (self.f)(&self.machine.state()).clone()
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        self.machine.traverse(edge);
    }
}

///
/// MappedTransitionMachine allows for mapping Machine state type into another type while maintaining
/// the same semantics.
///
pub struct MappedTransitionMachine<T, M, F, G> {
    pub t: PhantomData<T>,
    pub machine: M,
    pub f: F,
    pub g: G,
}

impl<M, F, G, T, U, V> Machine<T> for MappedTransitionMachine<T, M, F, G>
where
    M: Machine<T, Transition = U>,
    F: Fn(U) -> V,
    G: Fn(V) -> Option<U>,
    V: Clone,
{
    type Transition = V;

    fn edges(&self) -> impl Iterator<Item = Self::Transition> {
        self.machine.edges().map(|e| (self.f)(e))
    }

    fn state(&mut self) -> T {
        self.machine.state()
    }

    fn traverse(&mut self, edge: &Self::Transition) {
        if let Some(edge) = (self.g)(edge.clone()) {
            self.machine.traverse(&edge);
        }
    }
}
