use core::marker::PhantomData;

use crate::machine::Machine;

// Small helper so edges() can return either Empty or Map without boxing.
#[derive(Clone)]
enum EitherIter<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Iterator for EitherIter<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::Left(it) => it.next(),
            EitherIter::Right(it) => it.next(),
        }
    }
}

/// second: run machine on the 2nd component, pass through/remember A.
#[derive(Debug, Clone, Copy)]
pub struct SecondMachine<T: Send + Sync, I: Send, A: Send, M: Machine<I, T>> {
    t: PhantomData<T>,
    a: PhantomData<A>,
    i: PhantomData<I>,
    machine: M,
    last_a: Option<A>,
    cached_state: Option<(A, T)>,
}

impl<T: Send + Sync, I: Send, A: Send, M: Machine<I, T>> SecondMachine<T, I, A, M> {
    pub fn new(machine: M) -> Self {
        Self {
            t: PhantomData,
            a: PhantomData,
            i: PhantomData,
            machine,
            last_a: None,
            cached_state: None,
        }
    }

    fn invalidate_cache(&mut self) {
        self.cached_state = None;
    }
}

impl<T: Send + Sync, I: Send + Sync + Clone, A: Send + Sync, M> Machine<(A, I), (A, T)>
    for SecondMachine<T, I, A, M>
where
    M: Machine<I, T>,
    A: Clone,
    T: Clone,
{
    fn edges(&self) -> impl Iterator<Item = (A, I)> {
        match &self.last_a {
            None => EitherIter::Left(core::iter::empty()),
            Some(a0) => {
                let a0 = a0.clone();
                EitherIter::Right(self.machine.edges().map(move |tr| (a0.clone(), tr)))
            }
        }
    }

    fn state(&mut self) -> &(A, T) {
        if self.cached_state.is_none() {
            let a = self
                .last_a
                .as_ref()
                .expect("second(): state() called before any transitions")
                .clone();
            let t = self.machine.state().clone();
            self.cached_state = Some((a, t));
        }
        self.cached_state.as_ref().unwrap()
    }

    fn traverse(&mut self, edge: (A, I)) {
        let (a, tr) = edge;
        self.last_a = Some(a);
        self.machine.traverse(tr);
        self.invalidate_cache();
    }

    fn traverse_ref(&mut self, edge: &(A, I)) {
        self.last_a = Some(edge.0.clone());
        self.machine.traverse_ref(&edge.1);
        self.invalidate_cache();
    }
}

/// first: run machine on the 1st component, pass through/remember B.
#[derive(Debug, Clone, Copy)]
pub struct FirstMachine<T: Send + Sync, I: Send, B: Send, M: Machine<I, T>> {
    t: PhantomData<T>,
    b: PhantomData<B>,
    i: PhantomData<I>,
    machine: M,
    last_b: Option<B>,
    cached_state: Option<(T, B)>,
}

impl<T: Send + Sync, I: Send, B: Send, M: Machine<I, T>> FirstMachine<T, I, B, M> {
    pub fn new(machine: M) -> Self {
        Self {
            t: PhantomData,
            b: PhantomData,
            i: PhantomData,
            machine,
            last_b: None,
            cached_state: None,
        }
    }

    fn invalidate_cache(&mut self) {
        self.cached_state = None;
    }
}

impl<T: Send + Sync, I: Send + Sync + Clone, B: Send + Sync, M> Machine<(I, B), (T, B)>
    for FirstMachine<T, I, B, M>
where
    M: Machine<I, T>,
    B: Clone,
    T: Clone,
{
    fn edges(&self) -> impl Iterator<Item = (I, B)> {
        match &self.last_b {
            None => EitherIter::Left(core::iter::empty()),
            Some(b0) => {
                let b0 = b0.clone();
                EitherIter::Right(self.machine.edges().map(move |tr| (tr, b0.clone())))
            }
        }
    }

    fn state(&mut self) -> &(T, B) {
        if self.cached_state.is_none() {
            let t = self.machine.state().clone();
            let b = self
                .last_b
                .as_ref()
                .expect("first(): state() called before any transitions")
                .clone();
            self.cached_state = Some((t, b));
        }
        self.cached_state.as_ref().unwrap()
    }

    fn traverse(&mut self, edge: (I, B)) {
        let (tr, b) = edge;
        self.last_b = Some(b);
        self.machine.traverse(tr);
        self.invalidate_cache();
    }

    fn traverse_ref(&mut self, edge: &(I, B)) {
        self.last_b = Some(edge.1.clone());
        self.machine.traverse_ref(&edge.0);
        self.invalidate_cache();
    }
}

pub fn second<T: Send + Sync, I: Send + Sync + Clone, A: Send + Sync, M: Machine<I, T>>(
    machine: M,
) -> impl Machine<(A, I), (A, T)>
where
    A: Clone,
    T: Clone,
{
    SecondMachine::<T, I, A, M>::new(machine)
}

pub fn first<T: Send + Sync, I: Send + Sync + Clone, B: Send + Sync, M: Machine<I, T>>(
    machine: M,
) -> impl Machine<(I, B), (T, B)>
where
    B: Clone,
    T: Clone,
{
    FirstMachine::<T, I, B, M>::new(machine)
}
