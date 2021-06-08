//TODO: also make it/them such Stepper can go to a custom domainscale (they should still start at 0 for ease of use)
//TODO: create derives for Interpolation and Curve etc(?) -> https://github.com/rust-lang/rfcs/issues/1024
//TODO: make f64 the default input for Curves! -> this may reduce the need of structs with <f64,_,_,_>
//TODO: is Extrapolation as a marker trait also an idea?
use core::marker::PhantomData;
use core::ops::Range;

mod generator;
mod list;
mod space;

pub use generator::{Generator, Interpolation, Curve, DiscreteGenerator, Extract, Stepper};
pub use list::{Equidistant, ConstEquidistant, SortedList, NonEmptyGenerator, SortedGenerator, NonEmpty, Sorted};
pub use space::{Space, DynSpace, ConstSpace};


// Instead of using a collectionwrapper which can be bothersome, we want to implement generator traits
// for the collection itself. However because rust does not have GAT yet, we have to implement them
// one by one, as we can not implement it generically
//
// If one wants to use it's own collection, they have to implement the Generator trait itself.

impl<T: Copy> Generator<usize> for Vec<T> {
    type Output = T;
    fn gen(&self, input: usize) -> Self::Output {
        self[input]
    }
}

impl<T: Copy> DiscreteGenerator for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

// temporary hack
//-> Only implement SortedGenerator for Sorted<Vec<R>>
//-> Only implement NonEmptyGenerator for NonEmpty<Vec<R>>
//-> Only implement SortedList for Sorted<NonEmpty<Vec<R>>> and NonEmpty<Sorted<Vec<R>>>
impl<R: Copy> SortedGenerator for Vec<R> {}
impl<R: Copy> NonEmptyGenerator for Vec<R> {}
impl<R: Copy> SortedList for Vec<R> {}

/// A stack of values or generators
impl<G,I> Generator<(usize, I)> for Vec<G>
where G: Generator<I>
{
    type Output = G::Output;
    fn gen(&self, input: (usize, I)) -> Self::Output {
        self[input.0].gen(input.1)
    }
}

impl<T: Copy, const N: usize> Generator<usize> for [T;N] {
    type Output = T;
    fn gen(&self, input: usize) -> Self::Output {
        self[input]
    }
}

impl<T: Copy, const N: usize> DiscreteGenerator for [T;N] {
    fn len(&self) -> usize {
        N
    }
}

// temporary hack
impl<R: Copy, const N: usize> SortedGenerator for [R;N] {}
impl<R: Copy, const N: usize> NonEmptyGenerator for [R;N] {}
impl<R: Copy, const N: usize> SortedList for [R;N] {}

/// A stack of values or generators
impl<G,I, const N: usize> Generator<(usize, I)> for [G;N]
where G: Generator<I>
{
    type Output = G::Output;
    fn gen(&self, input: (usize, I)) -> Self::Output {
        self[input.0].gen(input.1)
    }
}
