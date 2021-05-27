//TODO: also make it/them such Stepper can go to a custom domainscale (they should still start at 0 for ease of use)
//TODO: create derives for Interpolation and Curve etc(?) -> https://github.com/rust-lang/rfcs/issues/1024
//TODO: make f64 the default input for Curves! -> this may reduce the need of structs with <f64,_,_,_>
//TODO: is Extrapolation as a marker trait also an idea?
use core::marker::PhantomData;
use core::ops::Range;

mod generator;
mod list;
mod space;

pub use generator::{Generator, Interpolation, Curve, FiniteGenerator, Extract, Stepper};
pub use list::{Equidistant, ConstEquidistant, SortedList};
pub use space::{Space, DynSpace, ConstSpace};


// Instead of using a collectionwrapper which can be bothersome, we want to implement generator traits
// for the collection itself. However because rust does not have GAT yet, we have to implement them
// one by one, as we can not implement it generically
//
// If one wants to use it's own collection, they have to implement the Generator trait itself.

impl<T: Copy> Generator<usize> for Vec<T> {
    type Output = T;
    fn get(&self, input: usize) -> Self::Output {
        self[input]
    }
}

impl<T: Copy> FiniteGenerator for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

impl<R: Copy> SortedList<R> for Vec<R> {}

impl<T: Copy, const N: usize> Generator<usize> for [T;N] {
    type Output = T;
    fn get(&self, input: usize) -> Self::Output {
        self[input]
    }
}

impl<T: Copy, const N: usize> FiniteGenerator for [T;N] {
    fn len(&self) -> usize {
        N
    }
}

impl<R: Copy, const N: usize> SortedList<R> for [R;N] {}


// ///CollectionWrapper acts like a stack of interpolations -> we try this first!
// impl<E,G,I> Generator<(usize,I)> for CollectionWrapper<E,G>
// where
//     E: AsRef<[G]>,
//     G: Generator<I>
// {
//     type Output = G::Output;
//     fn get(&self, input: (usize,I)) -> Self::Output {
//         self.0.as_ref()[input.0].get(input.1)
//     }
// }
