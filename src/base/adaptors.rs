use num_traits::real::Real;
use core::ops::{Add, Mul, RangeBounds, Bound};
use crate::{Generator, Interpolation, Curve, DiscreteGenerator, ConstDiscreteGenerator};

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

/// Struct which composite two generator together to act as one generator.
///
/// This `struct` is created by [`Generator::composite`]. See its documentation for more.
#[derive(Clone, Copy, Debug)]
pub struct Composition<A,B>(A,B);

impl<A,B> Composition<A,B>{
    /// Creates a stacked generator, working similar like the `zip` method of iterators.
    pub fn new(first: A, second: B) -> Self {
        Composition(first, second)
    }
}

impl<A,B,T> Generator<T> for Composition<A,B>
where
    A: Interpolation<T>,
    B: Interpolation<A::Output>
{
    type Output = B::Output;
    fn gen(&self, scalar: T) -> Self::Output {
        self.1.gen(self.0.gen(scalar))
    }
}

impl<A,B,T> Interpolation<T> for Composition<A,B>
where
    A: Interpolation<T>,
    B: Interpolation<A::Output>
{}

impl<A,B,R> Curve<R> for Composition<A,B>
where
    A: Curve<R>,
    B: Interpolation<A::Output>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        self.0.domain()
    }
}

/// DiscreteGenerator Adaptor which takes two generators with output S and T and stacks them, such that the output is (T,R).
///
/// This `struct` is created by [`Generator::stack]. See its documentation for more.
#[derive(Debug, Copy, Clone)]
pub struct Stack<G,H>(G,H);

impl<G,H> Stack<G,H>{
    /// Creates a stacked generator, working similar like the `zip` method of iterators.
    pub fn new(first: G, second: H) -> Self {
        Stack(first,second)
    }
}

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

/// DiscreteGenerator Adaptor which repeats the underlying elements.
#[derive(Debug, Copy, Clone)]
pub struct Repeat<G>(G);

impl<G> Repeat<G>{
    /// Repeat a given DiscreteGenerator pseudo-endlessly.
    ///
    /// In reality this adaptpor repeats the underlying elements until `usize::MAX` is reached.
    pub fn new(gen: G) -> Self {
        Repeat(gen)
    }
}

impl<G> Generator<usize> for Repeat<G>
where
    G: DiscreteGenerator,
{
    type Output = G::Output;
    fn gen(&self, input: usize) -> Self::Output {
        self.0.gen(input % self.0.len())
    }
}

impl<G> DiscreteGenerator for Repeat<G>
where
    G: DiscreteGenerator
{
    fn len(&self) -> usize {
        usize::MAX
    }
}

impl<G> ConstDiscreteGenerator<{usize::MAX}> for Repeat<G>
where G: DiscreteGenerator
{}

/// DiscreteGenerator Adaptor which repeats a fixed amount of first elements.
#[derive(Debug, Copy, Clone)]
pub struct Wrap<G>{
    inner: G,
    n: usize,
}

impl<G> Wrap<G>{
    /// Wrap the first `n` elements to the end.
    pub fn new(gen: G, n: usize) -> Self {
        Wrap{
            inner: gen,
            n,
        }
    }
}

impl<G> Generator<usize> for Wrap<G>
where
    G: DiscreteGenerator,
{
    type Output = G::Output;
    fn gen(&self, input: usize) -> Self::Output {
        self.inner.gen(input % self.inner.len())
    }
}

impl<G> DiscreteGenerator for Wrap<G>
where
    G: DiscreteGenerator
{
    fn len(&self) -> usize {
        self.inner.len() + self.n
    }
}
