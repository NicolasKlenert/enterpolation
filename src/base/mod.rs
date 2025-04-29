mod adaptors;
mod generator;
mod list;
mod space;

// These get re-exported at the library level.
#[allow(unreachable_pub)]
pub use adaptors::{Clamp, Composite, Repeat, Slice, Stack, TransformInput, Wrap};
#[allow(unreachable_pub)]
pub use generator::{
    ConstDiscreteGenerator, Curve, DiscreteGenerator, Extract, Generator, Stepper,
};
#[allow(unreachable_pub)]
pub use list::{ConstEquidistant, Equidistant, NotSorted, Sorted, SortedGenerator};
#[allow(unreachable_pub)]
#[cfg(feature = "std")]
pub use space::DynSpace;
#[allow(unreachable_pub)]
pub use space::{ConstSpace, Space};

#[cfg(feature = "std")]
impl<T: Copy> Generator<usize> for Vec<T> {
    type Output = T;
    fn interpolate(&self, input: usize) -> Self::Output {
        self[input]
    }
}
#[cfg(feature = "std")]
impl<T: Copy> DiscreteGenerator for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

// /// A stack of values or generators
// #[cfg(feature = "std")]
// impl<G,I> Generator<(usize, I)> for Vec<G>
// where G: Generator<I>
// {
//     type Output = G::Output;
//     fn gen(&self, input: (usize, I)) -> Self::Output {
//         self[input.0].gen(input.1)
//     }
// }

impl<T: Copy> Generator<usize> for &[T] {
    type Output = T;
    fn interpolate(&self, input: usize) -> Self::Output {
        self[input]
    }
}

impl<T: Copy> DiscreteGenerator for &[T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T: Copy, const N: usize> Generator<usize> for [T; N] {
    type Output = T;
    fn interpolate(&self, input: usize) -> Self::Output {
        self[input]
    }
}

impl<T: Copy, const N: usize> DiscreteGenerator for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T: Copy, const N: usize> ConstDiscreteGenerator<N> for [T; N] {}

// /// A stack of values or generators
// impl<G,I, const N: usize> Generator<(usize, I)> for [G;N]
// where G: Generator<I>
// {
//     type Output = G::Output;
//     fn gen(&self, input: (usize, I)) -> Self::Output {
//         self[input.0].gen(input.1)
//     }
// }
