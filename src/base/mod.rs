//TODO: create derives for Interpolation and Curve etc(?) -> https://github.com/rust-lang/rfcs/issues/1024
//TODO: is Extrapolation as a marker trait also an idea?

mod generator;
mod list;
mod space;

pub use generator::{Generator, Interpolation, Curve, DiscreteGenerator, ConstDiscreteGenerator, Extract, Stepper, Take};
pub use list::{Equidistant, ConstEquidistant, SortedGenerator, Sorted, NotSorted};
pub use space::{Space, DynSpace, ConstSpace};

use num_traits::real::Real;

/// Generator Adaptor which takes two generators with Output S and T and stacks them, such that the output is (T,R).
pub struct Stack<G,H>(G,H);

impl<G,H,Input> Generator<Input> for Stack<G,H>
where
    G: Generator<Input>,
    H: Generator<Input>,
    Input: Copy,
{
    type Output = (G::Output, H::Output);
    fn gen(&self, input: Input) -> Self::Output {
        (self.0.gen(input), self.1.gen(input))
    }
}

impl<G,H> DiscreteGenerator for Stack<G,H>
where
    G: DiscreteGenerator,
    H: DiscreteGenerator,
{
    fn len(&self) -> usize {
        self.0.len().min(self.1.len())
    }
}

impl<G,H, const N: usize> ConstDiscreteGenerator<N> for Stack<G,H>
where
    G: ConstDiscreteGenerator<N>,
    H: ConstDiscreteGenerator<N>,
{}

impl<G,H,Input> Interpolation<Input> for Stack<G,H>
where
    G: Interpolation<Input>,
    H: Interpolation<Input>,
    Input: Copy
{}

impl<G,H,R> Curve<R> for Stack<G,H>
where
    G: Curve<R>,
    H: Curve<R>,
    R: Real
{
    fn domain(&self) -> [R; 2] {
        let first = self.0.domain();
        let second = self.1.domain();
        [first[0].max(second[0]),first[1].min(second[1])]
    }
}

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
