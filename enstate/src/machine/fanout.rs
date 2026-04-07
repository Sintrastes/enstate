use core::marker::PhantomData;

use crate::machine::Machine;

#[derive(Debug, Clone, Copy)]
pub struct FanoutMachine<I, A, B, M1, M2> {
    a: PhantomData<A>,
    b: PhantomData<B>,
    i: PhantomData<I>,
    machine1: M1,
    machine2: M2,
    cached_state: Option<(A, B)>,
}

impl<I, A, B, M1, M2> FanoutMachine<I, A, B, M1, M2> {
    #[inline]
    pub fn new(machine1: M1, machine2: M2) -> Self {
        Self {
            a: PhantomData,
            b: PhantomData,
            i: PhantomData,
            machine1,
            machine2,
            cached_state: None,
        }
    }

    #[inline]
    fn invalidate_cache(&mut self) {
        self.cached_state = None;
    }
}

impl<I: Send + Sync + Clone, A, B, M1, M2> Machine<I, (A, B)> for FanoutMachine<I, A, B, M1, M2>
where
    M1: Machine<I, A>,
    M2: Machine<I, B>,
    A: Clone + Send + Sync,
    B: Clone + Send + Sync,
{
    #[inline]
    fn edges(&self) -> impl Iterator<Item = I> {
        // Union of possible transitions. Duplicates are allowed.
        self.machine1.edges().chain(self.machine2.edges())
    }

    #[inline]
    fn state(&mut self) -> &(A, B) {
        if self.cached_state.is_none() {
            // Materialize combined state once per “tick” (until traversed).
            let a = self.machine1.state().clone();
            let b = self.machine2.state().clone();
            self.cached_state = Some((a, b));
        }
        self.cached_state.as_ref().unwrap()
    }

    #[inline]
    fn traverse(&mut self, edge: I) {
        self.traverse_ref(&edge);
    }

    #[inline]
    fn traverse_ref(&mut self, edge: &I) {
        self.machine1.traverse_ref(edge);
        self.machine2.traverse_ref(edge);
        self.invalidate_cache();
    }
}

/// Implementation of Haskell's &&& fanout operator.
#[inline]
pub fn fanout<I: Send + Sync + Clone, A, B, M1, M2>(m1: M1, m2: M2) -> impl Machine<I, (A, B)>
where
    M1: Machine<I, A>,
    M2: Machine<I, B>,
    A: Clone + Send + Sync,
    B: Clone + Send + Sync,
{
    FanoutMachine::new(m1, m2)
}

#[derive(Clone)]
pub struct FanoutMachine3<I, A, B, C, M1, M2, M3> {
    a: PhantomData<A>,
    b: PhantomData<B>,
    c: PhantomData<C>,
    i: PhantomData<I>,
    machine1: M1,
    machine2: M2,
    machine3: M3,
    cached_state: Option<(A, B, C)>,
}

impl<I, A, B, C, M1, M2, M3> FanoutMachine3<I, A, B, C, M1, M2, M3> {
    #[inline]
    pub fn new(machine1: M1, machine2: M2, machine3: M3) -> Self {
        Self {
            a: PhantomData,
            b: PhantomData,
            c: PhantomData,
            i: PhantomData,
            machine1,
            machine2,
            machine3,
            cached_state: None,
        }
    }

    #[inline]
    fn invalidate_cache(&mut self) {
        self.cached_state = None;
    }
}

impl<I: Send + Sync + Clone, A, B, C, M1, M2, M3> Machine<I, (A, B, C)>
    for FanoutMachine3<I, A, B, C, M1, M2, M3>
where
    M1: Machine<I, A>,
    M2: Machine<I, B>,
    M3: Machine<I, C>,
    A: Clone + Send + Sync,
    B: Clone + Send + Sync,
    C: Clone + Send + Sync,
{
    #[inline]
    fn edges(&self) -> impl Iterator<Item = I> {
        // Union of possible transitions. Duplicates are allowed.
        self.machine1.edges().chain(self.machine2.edges())
    }

    #[inline]
    fn state(&mut self) -> &(A, B, C) {
        if self.cached_state.is_none() {
            // Materialize combined state once per “tick” (until traversed).
            let a = self.machine1.state().clone();
            let b = self.machine2.state().clone();
            let c = self.machine3.state().clone();
            self.cached_state = Some((a, b, c));
        }
        self.cached_state.as_ref().unwrap()
    }

    #[inline]
    fn traverse(&mut self, edge: I) {
        self.traverse_ref(&edge);
    }

    #[inline]
    fn traverse_ref(&mut self, edge: &I) {
        self.machine1.traverse_ref(edge);
        self.machine2.traverse_ref(edge);
        self.machine3.traverse_ref(edge);
        self.invalidate_cache();
    }
}

#[inline]
pub fn fanout3<I: Send + Sync + Clone, A, B, C, M1, M2, M3>(
    m1: M1,
    m2: M2,
    m3: M3,
) -> impl Machine<I, (A, B, C)>
where
    M1: Machine<I, A>,
    M2: Machine<I, B>,
    M3: Machine<I, C>,
    A: Clone + Send + Sync,
    B: Clone + Send + Sync,
    C: Clone + Send + Sync,
{
    FanoutMachine3::new(m1, m2, m3)
}

#[derive(Clone)]
pub struct FanoutMachine4<I, A, B, C, D, M1, M2, M3, M4> {
    a: PhantomData<A>,
    b: PhantomData<B>,
    c: PhantomData<C>,
    d: PhantomData<D>,
    i: PhantomData<I>,
    machine1: M1,
    machine2: M2,
    machine3: M3,
    machine4: M4,
    cached_state: Option<(A, B, C, D)>,
}

impl<I, A, B, C, D, M1, M2, M3, M4> FanoutMachine4<I, A, B, C, D, M1, M2, M3, M4> {
    #[inline]
    pub fn new(machine1: M1, machine2: M2, machine3: M3, machine4: M4) -> Self {
        Self {
            a: PhantomData,
            b: PhantomData,
            c: PhantomData,
            d: PhantomData,
            i: PhantomData,
            machine1,
            machine2,
            machine3,
            machine4,
            cached_state: None,
        }
    }

    #[inline]
    fn invalidate_cache(&mut self) {
        self.cached_state = None;
    }
}

impl<I: Send + Sync + Clone, A, B, C, D, M1, M2, M3, M4> Machine<I, (A, B, C, D)>
    for FanoutMachine4<I, A, B, C, D, M1, M2, M3, M4>
where
    M1: Machine<I, A>,
    M2: Machine<I, B>,
    M3: Machine<I, C>,
    M4: Machine<I, D>,
    A: Clone + Send + Sync,
    B: Clone + Send + Sync,
    C: Clone + Send + Sync,
    D: Clone + Send + Sync,
{
    #[inline]
    fn edges(&self) -> impl Iterator<Item = I> {
        // Union of possible transitions. Duplicates are allowed.
        self.machine1
            .edges()
            .chain(self.machine2.edges())
            .chain(self.machine3.edges())
            .chain(self.machine4.edges())
    }

    #[inline]
    fn state(&mut self) -> &(A, B, C, D) {
        if self.cached_state.is_none() {
            // Materialize combined state once per “tick” (until traversed).
            let a = self.machine1.state().clone();
            let b = self.machine2.state().clone();
            let c = self.machine3.state().clone();
            let d = self.machine4.state().clone();
            self.cached_state = Some((a, b, c, d));
        }
        self.cached_state.as_ref().unwrap()
    }

    #[inline]
    fn traverse(&mut self, edge: I) {
        self.traverse_ref(&edge);
    }

    #[inline]
    fn traverse_ref(&mut self, edge: &I) {
        self.machine1.traverse_ref(edge);
        self.machine2.traverse_ref(edge);
        self.machine3.traverse_ref(edge);
        self.machine4.traverse_ref(edge);
        self.invalidate_cache();
    }
}

#[inline]
pub fn fanout4<I: Send + Sync + Clone, A: Sync, B: Sync, C: Sync, D: Sync, M1, M2, M3, M4>(
    m1: M1,
    m2: M2,
    m3: M3,
    m4: M4,
) -> impl Machine<I, (A, B, C, D)> + Clone
where
    M1: Machine<I, A> + Clone,
    M2: Machine<I, B> + Clone,
    M3: Machine<I, C> + Clone,
    M4: Machine<I, D> + Clone,
    A: Clone + Send,
    B: Clone + Send,
    C: Clone + Send,
    D: Clone + Send,
{
    FanoutMachine4::new(m1, m2, m3, m4)
}

// ------------- Fanout 5 ---------------------- //
//
//
#[derive(Clone)]
pub struct FanoutMachine5<I, A, B, C, D, E, M1, M2, M3, M4, M5> {
    a: PhantomData<A>,
    b: PhantomData<B>,
    c: PhantomData<C>,
    d: PhantomData<D>,
    e: PhantomData<E>,
    i: PhantomData<I>,
    machine1: M1,
    machine2: M2,
    machine3: M3,
    machine4: M4,
    machine5: M5,
    cached_state: Option<(A, B, C, D, E)>,
}

impl<I, A, B, C, D, E, M1, M2, M3, M4, M5> FanoutMachine5<I, A, B, C, D, E, M1, M2, M3, M4, M5> {
    #[inline]
    pub fn new(machine1: M1, machine2: M2, machine3: M3, machine4: M4, machine5: M5) -> Self {
        Self {
            a: PhantomData,
            b: PhantomData,
            c: PhantomData,
            d: PhantomData,
            e: PhantomData,
            i: PhantomData,
            machine1,
            machine2,
            machine3,
            machine4,
            machine5,
            cached_state: None,
        }
    }

    #[inline]
    fn invalidate_cache(&mut self) {
        self.cached_state = None;
    }
}

impl<I: Send + Sync + Clone, A, B, C, D, E, M1, M2, M3, M4, M5> Machine<I, (A, B, C, D, E)>
    for FanoutMachine5<I, A, B, C, D, E, M1, M2, M3, M4, M5>
where
    M1: Machine<I, A>,
    M2: Machine<I, B>,
    M3: Machine<I, C>,
    M4: Machine<I, D>,
    M5: Machine<I, E>,
    A: Clone + Send + Sync,
    B: Clone + Send + Sync,
    C: Clone + Send + Sync,
    D: Clone + Send + Sync,
    E: Clone + Send + Sync,
{
    #[inline]
    fn edges(&self) -> impl Iterator<Item = I> {
        // Union of possible transitions. Duplicates are allowed.
        self.machine1
            .edges()
            .chain(self.machine2.edges())
            .chain(self.machine3.edges())
            .chain(self.machine4.edges())
            .chain(self.machine5.edges())
    }

    #[inline]
    fn state(&mut self) -> &(A, B, C, D, E) {
        if self.cached_state.is_none() {
            // Materialize combined state once per “tick” (until traversed).
            let a = self.machine1.state().clone();
            let b = self.machine2.state().clone();
            let c = self.machine3.state().clone();
            let d = self.machine4.state().clone();
            let e = self.machine5.state().clone();
            self.cached_state = Some((a, b, c, d, e));
        }
        self.cached_state.as_ref().unwrap()
    }

    #[inline]
    fn traverse(&mut self, edge: I) {
        self.traverse_ref(&edge);
    }

    #[inline]
    fn traverse_ref(&mut self, edge: &I) {
        self.machine1.traverse_ref(edge);
        self.machine2.traverse_ref(edge);
        self.machine3.traverse_ref(edge);
        self.machine4.traverse_ref(edge);
        self.machine5.traverse_ref(edge);
        self.invalidate_cache();
    }
}

#[inline]
pub fn fanout5<
    I: Send + Sync + Clone,
    A: Sync,
    B: Sync,
    C: Sync,
    D: Sync,
    E: Sync,
    M1,
    M2,
    M3,
    M4,
    M5,
>(
    m1: M1,
    m2: M2,
    m3: M3,
    m4: M4,
    m5: M5,
) -> impl Machine<I, (A, B, C, D, E)> + Clone
where
    M1: Machine<I, A> + Clone,
    M2: Machine<I, B> + Clone,
    M3: Machine<I, C> + Clone,
    M4: Machine<I, D> + Clone,
    M5: Machine<I, E> + Clone,
    A: Clone + Send,
    B: Clone + Send,
    C: Clone + Send,
    D: Clone + Send,
    E: Clone + Send,
{
    FanoutMachine5::new(m1, m2, m3, m4, m5)
}
