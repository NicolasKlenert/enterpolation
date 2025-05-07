mod adaptors;
mod list;
mod signal;
mod space;

// These get re-exported at the library level.
#[allow(unreachable_pub)]
pub use adaptors::{Clamp, Composite, Repeat, Slice, Stack, TransformInput, Wrap};
#[allow(unreachable_pub)]
pub use list::{ConstEquidistant, Equidistant, NotSorted, Sorted, SortedChain};
#[allow(unreachable_pub)]
pub use signal::{Chain, ConstChain, Curve, Extract, Signal, Stepper};
#[allow(unreachable_pub)]
#[cfg(feature = "std")]
pub use space::DynSpace;
#[allow(unreachable_pub)]
pub use space::{ConstSpace, Space};

#[cfg(feature = "std")]
impl<T: Copy> Signal<usize> for Vec<T> {
    type Output = T;
    fn eval(&self, input: usize) -> Self::Output {
        self[input]
    }
}
#[cfg(feature = "std")]
impl<T: Copy> Chain for Vec<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

// /// A stack of values or signals
// #[cfg(feature = "std")]
// impl<G,I> Signal<(usize, I)> for Vec<G>
// where G: Signal<I>
// {
//     type Output = G::Output;
//     fn eval(&self, input: (usize, I)) -> Self::Output {
//         self[input.0].eval(input.1)
//     }
// }

impl<T: Copy> Signal<usize> for &[T] {
    type Output = T;
    fn eval(&self, input: usize) -> Self::Output {
        self[input]
    }
}

impl<T: Copy> Chain for &[T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}

impl<T: Copy, const N: usize> Signal<usize> for [T; N] {
    type Output = T;
    fn eval(&self, input: usize) -> Self::Output {
        self[input]
    }
}

impl<T: Copy, const N: usize> Chain for [T; N] {
    fn len(&self) -> usize {
        N
    }
}

impl<T: Copy, const N: usize> ConstChain<N> for [T; N] {}

// /// A stack of values or signals
// impl<G,I, const N: usize> Signal<(usize, I)> for [G;N]
// where G: Signal<I>
// {
//     type Output = G::Output;
//     fn eval(&self, input: (usize, I)) -> Self::Output {
//         self[input.0].eval(input.1)
//     }
// }
