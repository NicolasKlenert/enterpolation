//TODO: create derives for Interpolation and Curve etc(?) -> https://github.com/rust-lang/rfcs/issues/1024
//TODO: is Extrapolation as a marker trait also an idea?

mod generator;
mod adaptors;
mod list;
mod space;

// These get re-exported at the library level.
#[allow(unreachable_pub)]
pub use generator::{Generator, Interpolation, Curve, DiscreteGenerator, ConstDiscreteGenerator, Extract, Stepper, Take};
#[allow(unreachable_pub)]
pub use adaptors::{Composition, Stack, TransformInput, Slice, Repeat, Wrap};
#[allow(unreachable_pub)]
pub use list::{Equidistant, ConstEquidistant, SortedGenerator, Sorted, NotSorted};
#[allow(unreachable_pub)]
pub use space::{Space, ConstSpace};
#[allow(unreachable_pub)]
#[cfg(feature = "std")]
pub use space::DynSpace;

#[cfg(feature = "std")]
impl<T: Copy> Generator<usize> for Vec<T> {
    type Output = T;
    fn gen(&self, input: usize) -> Self::Output {
        self[input]
    }
}
#[cfg(feature = "std")]
impl<T: Copy> DiscreteGenerator for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

/// A stack of values or generators
#[cfg(feature = "std")]
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

impl<T:Copy, const N: usize> ConstDiscreteGenerator<N> for [T;N] {}

/// A stack of values or generators
impl<G,I, const N: usize> Generator<(usize, I)> for [G;N]
where G: Generator<I>
{
    type Output = G::Output;
    fn gen(&self, input: (usize, I)) -> Self::Output {
        self[input.0].gen(input.1)
    }
}
