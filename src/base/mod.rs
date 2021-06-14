//TODO: create derives for Interpolation and Curve etc(?) -> https://github.com/rust-lang/rfcs/issues/1024
//TODO: is Extrapolation as a marker trait also an idea?

mod generator;
mod list;
mod space;

pub use generator::{Generator, Interpolation, Curve, DiscreteGenerator, Extract, Stepper};
pub use list::{Equidistant, ConstEquidistant, SortedGenerator, Sorted, NotSorted};
pub use space::{Space, DynSpace, ConstSpace};

/// Trait for possible heterogen structures which can be disassembled step by step.
pub trait Composite<Head>{
    /// The Tail (everything except the head) of the composite type.
    type Tail;
    /// Split the composite type to get its head and tail.
    fn split(self) -> (Head, Self::Tail);
}

impl<H,T> Composite<H> for (H,T){
    type Tail = T;
    fn split(self) -> (H, Self::Tail) {
        self
    }
}

// impl<H> Composite<H> for H {
//     type Tail = ();
//     fn split(self) -> (H, Self::Tail){
//         (self, ())
//     }
// }

// Not yet possible
// impl<H,T, const N: usize> Composite<H> for [H;N] {
//     type Tail = [H; N-1];
//     fn split(self) -> (H, Self::Tail) {
//         (self[0],)
//     }
// }

// Instead of using a collectionwrapper which can be bothersome, we want to implement generator traits
// for the collection itself. However because rust does not have GAT or specialization yet, we have to implement them
// one by one, as we can not implement it generically
//
// If one wants to use it's own collection, they have to implement the Generator trait itself.

// This would be needed to implement mutlivariate interpolations without any new struct.
// impl<T: Copy> Generator<()> for T {
//     type Output = T;
//     fn gen(&self, _input: ()) -> Self::Output {
//         *self
//     }
// }

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
// Only implement MinSizeGenerator<M> for [R;N] where M <= N.
// impl<R: Copy, const N: usize> MinSizeGenerator<2> for [R;N] {}

/// A stack of values or generators
impl<G,I, const N: usize> Generator<(usize, I)> for [G;N]
where G: Generator<I>
{
    type Output = G::Output;
    fn gen(&self, input: (usize, I)) -> Self::Output {
        self[input.0].gen(input.1)
    }
}
