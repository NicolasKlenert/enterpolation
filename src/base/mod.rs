//TODO: create derives for Interpolation and Curve etc(?) -> https://github.com/rust-lang/rfcs/issues/1024
//TODO: is Extrapolation as a marker trait also an idea?

mod generator;
mod list;
mod space;

// These get re-exported at the library level.
#[allow(unreachable_pub)]
pub use generator::{Generator, Interpolation, Curve, DiscreteGenerator, ConstDiscreteGenerator, Extract, Stepper, Take};
#[allow(unreachable_pub)]
pub use list::{Equidistant, ConstEquidistant, SortedGenerator, Sorted, NotSorted};
#[allow(unreachable_pub)]
pub use space::{Space, DynSpace, ConstSpace};

use num_traits::real::Real;
use core::ops::{Add, Mul, RangeBounds, Bound};

/// Acts like a slice of a curve
///
/// That is, a slice of a curve has the same domain as the curve itself but maps the domain onto the range given.
#[derive(Clone, Debug)]
pub struct Slice<'a,G,R>(TransformInput<&'a G, R, R>);

impl<'a,G,R> Slice<'a,G,R>
where
    G: Curve<R>,
    R: Real,
{
    /// Create a new slice of the given generator.
    ///
    /// It does not matter if the bounds itself are included or excluded as we assume a continuous curve.
    pub fn new<B>(gen: &'a G, bound: B) -> Self
    where B: RangeBounds<R>,
    {
        let [gen_start, gen_end] = gen.domain();
        let bound_start = match bound.start_bound() {
            Bound::Included(x) | Bound::Excluded(x) => *x,
            Bound::Unbounded => gen_start,
        };
        let bound_end = match bound.end_bound() {
            Bound::Included(x) | Bound::Excluded(x) => *x,
            Bound::Unbounded => gen_end,
        };
        let scale = (bound_end - bound_start) / (gen_end - gen_start);
        Slice(TransformInput::new(gen,bound_start - gen_start, scale))
    }
}

impl<G,R> Generator<R> for Slice<'_,G,R>
where
    G: Generator<R>,
    R: Real
{
    type Output = G::Output;
    fn gen(&self, input: R) -> Self::Output {
        self.0.gen(input)
    }
}

impl<G,R> Interpolation<R> for Slice<'_,G,R>
where
    G: Interpolation<R>,
    R: Real
{}

impl<G,R> Curve<R> for Slice<'_,G,R>
where
    G: Curve<R>,
    R: Real,
{
    fn domain(&self) -> [R;2]{
        self.0.domain()
    }
}

/// Struct which transforms the input before sending it to the underlying generator.
///
/// Both addition and multiplication is done. In regards to math operation priorities, multiplication is done first.
#[derive(Clone, Debug)]
pub struct TransformInput<G,A,M>{
    addition: A,
    multiplication: M,
    inner: G
}

impl<G,A,M> TransformInput<G,A,M>{
    /// Create a generic `TransformInput`.
    pub fn new(generator: G, addition: A, multiplication: M) -> Self {
        TransformInput {
            inner: generator,
            addition,
            multiplication,
        }
    }
}

impl<G,R> TransformInput<G,R,R>
where
    G: Curve<R>,
    R: Real,
{
    /// Transfrom an input such that the wrapped generator changes its domain from [0.0,1.0] to
    /// the domain wished for.
    pub fn normalized_to_domain(generator: G, start: R, end: R) -> Self {
        Self::new(generator, -start, (end - start).recip())
    }
}

impl<G,A,M,I> Generator<I> for TransformInput<G,A,M>
where
    I: Mul<M>,
    I::Output: Add<A>,
    A: Copy,
    M: Copy,
    G: Generator<<<I as Mul<M>>::Output as Add<A>>::Output>,
{
    type Output = G::Output;
    fn gen(&self, input: I) -> Self::Output {
        self.inner.gen(input * self.multiplication + self.addition)
    }
}

impl<G,A,M,I> Interpolation<I> for TransformInput<G,A,M>
where
    I: Mul<M>,
    I::Output: Add<A>,
    A: Copy,
    M: Copy,
    G: Interpolation<<<I as Mul<M>>::Output as Add<A>>::Output>,
{}

impl<G,R> Curve<R> for TransformInput<G,R,R>
where
    G: Curve<R>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        let orig = self.inner.domain();
        let start = (orig[0] - self.addition) / self.multiplication;
        let end = (orig[1] - self.addition) / self.multiplication;
        [start,end]
    }
}

/// Struct which chains two Interpolation together to one Interpolation.
///
/// This `struct` is created by [`Interpolation::chain`]. See its documentation for more.
#[derive(Clone, Debug)]
pub struct Chain<A,B>{
    first: A,
    second: B
}

impl<A,B,T> Generator<T> for Chain<A,B>
where
    A: Interpolation<T>,
    B: Interpolation<A::Output>
{
    type Output = B::Output;
    fn gen(&self, scalar: T) -> Self::Output {
        self.second.gen(self.first.gen(scalar))
    }
}

impl<A,B,T> Interpolation<T> for Chain<A,B>
where
    A: Interpolation<T>,
    B: Interpolation<A::Output>
{}

impl<A,B,R> Curve<R> for Chain<A,B>
where
    A: Curve<R>,
    B: Interpolation<A::Output>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        self.first.domain()
    }
}

/// Generator Adaptor which takes two generators with Output S and T and stacks them, such that the output is (T,R).
#[derive(Debug, Copy, Clone)]
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
