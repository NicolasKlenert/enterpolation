use crate::{Chain, ConstChain, Curve, Signal};
use core::ops::{Add, Bound, Mul, RangeBounds};
use num_traits::clamp;
use num_traits::real::Real;

/// Wrapper for curves to clamp input to their domain.
///
/// This struct is constructed through the [`clamp()`] method of curves.
/// Please look there for more information.
///
/// [`clamp()`]: crate::Curve::clamp()
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Clamp<G>(G);

impl<G> Clamp<G> {
    /// Create a new `Clamp` struct.
    pub fn new(signal: G) -> Self {
        Clamp(signal)
    }
}

impl<G, R> Signal<R> for Clamp<G>
where
    G: Curve<R>,
    R: Real,
{
    type Output = G::Output;
    fn eval(&self, input: R) -> Self::Output {
        let [min, max] = self.domain();
        let clamped = clamp(input, min, max);
        self.0.eval(clamped)
    }
}

impl<G, R> Curve<R> for Clamp<G>
where
    G: Curve<R>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        self.0.domain()
    }
}

/// Acts like a slice of a curve.
///
/// That is, a slice of a curve has the same domain as the curve itself but maps the domain onto the range given.
///
/// This struct is created by the [`slice()`] method. Please look their for more information.
///
/// [`slice()`]: crate::Curve::slice()
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Slice<G, R>(TransformInput<G, R, R>);

impl<G, R> Slice<G, R>
where
    G: Curve<R>,
    R: Real,
{
    /// Create a new slice of the given signal.
    ///
    /// It does not matter if the bounds itself are included or excluded as we assume a continuous curve.
    pub fn new<B>(signal: G, bound: B) -> Self
    where
        B: RangeBounds<R>,
    {
        let [signal_start, signal_end] = signal.domain();
        let bound_start = match bound.start_bound() {
            Bound::Included(x) | Bound::Excluded(x) => *x,
            Bound::Unbounded => signal_start,
        };
        let bound_end = match bound.end_bound() {
            Bound::Included(x) | Bound::Excluded(x) => *x,
            Bound::Unbounded => signal_end,
        };
        let scale = (bound_end - bound_start) / (signal_end - signal_start);
        Slice(TransformInput::new(
            signal,
            bound_start - signal_start,
            scale,
        ))
    }
}

impl<G, R> Signal<R> for Slice<G, R>
where
    G: Signal<R>,
    R: Real,
{
    type Output = G::Output;
    fn eval(&self, input: R) -> Self::Output {
        self.0.eval(input)
    }
}

impl<G, R> Curve<R> for Slice<G, R>
where
    G: Curve<R>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        self.0.inner.domain()
    }
}

/// Struct which transforms the input before sending it to the underlying signal.
///
/// Both addition and multiplication is done. In regards to math operation priorities, multiplication is done first.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TransformInput<G, A, M> {
    addition: A,
    multiplication: M,
    inner: G,
}

impl<G, A, M> TransformInput<G, A, M> {
    /// Create a generic `TransformInput`.
    pub fn new(signal: G, addition: A, multiplication: M) -> Self {
        TransformInput {
            inner: signal,
            addition,
            multiplication,
        }
    }
}

impl<G, R> TransformInput<G, R, R>
where
    G: Curve<R>,
    R: Real,
{
    /// Transform an input such that the wrapped signal changes its domain from [0.0,1.0] to
    /// the domain wished for.
    pub fn normalized_to_domain(signal: G, start: R, end: R) -> Self {
        Self::new(signal, -start, (end - start).recip())
    }
}

impl<G, A, M, I> Signal<I> for TransformInput<G, A, M>
where
    I: Mul<M>,
    I::Output: Add<A>,
    A: Copy,
    M: Copy,
    G: Signal<<<I as Mul<M>>::Output as Add<A>>::Output>,
{
    type Output = G::Output;
    fn eval(&self, input: I) -> Self::Output {
        self.inner.eval(input * self.multiplication + self.addition)
    }
}

impl<G, R> Curve<R> for TransformInput<G, R, R>
where
    G: Curve<R>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        let orig = self.inner.domain();
        let start = (orig[0] - self.addition) / self.multiplication;
        let end = (orig[1] - self.addition) / self.multiplication;
        [start, end]
    }
}

/// Struct which composite two signal together to act as one signal.
///
/// This `struct` is created by [`Signal::composite`]. See its documentation for more.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Composite<A, B>(A, B);

impl<A, B> Composite<A, B> {
    /// Creates a composite signal.
    pub fn new(first: A, second: B) -> Self {
        Composite(first, second)
    }
}

impl<A, B, T> Signal<T> for Composite<A, B>
where
    A: Signal<T>,
    B: Signal<A::Output>,
{
    type Output = B::Output;
    fn eval(&self, scalar: T) -> Self::Output {
        self.1.eval(self.0.eval(scalar))
    }
}

impl<A, B, R> Curve<R> for Composite<A, B>
where
    A: Curve<R>,
    B: Signal<A::Output>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        self.0.domain()
    }
}

/// Chain adaptor which stacks two signals.
///
/// That it, the struct holds two signals with output S and T and outputs (S,T).
///
/// This `struct` is created by [`Signal::stack]. See its documentation for more.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Stack<G, H>(G, H);

impl<G, H> Stack<G, H> {
    /// Creates a stacked signal, working similar like the `zip` method of iterators.
    pub fn new(first: G, second: H) -> Self {
        Stack(first, second)
    }
}

impl<G, H, Input> Signal<Input> for Stack<G, H>
where
    G: Signal<Input>,
    H: Signal<Input>,
    Input: Copy,
{
    type Output = (G::Output, H::Output);
    fn eval(&self, input: Input) -> Self::Output {
        (self.0.eval(input), self.1.eval(input))
    }
}

impl<G, H> Chain for Stack<G, H>
where
    G: Chain,
    H: Chain,
{
    fn len(&self) -> usize {
        self.0.len().min(self.1.len())
    }
}

impl<G, H, const N: usize> ConstChain<N> for Stack<G, H>
where
    G: ConstChain<N>,
    H: ConstChain<N>,
{
}

impl<G, H, R> Curve<R> for Stack<G, H>
where
    G: Curve<R>,
    H: Curve<R>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        let first = self.0.domain();
        let second = self.1.domain();
        [first[0].max(second[0]), first[1].min(second[1])]
    }
}

/// Chain Adaptor which repeats the underlying elements.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Repeat<G>(G);

impl<G> Repeat<G> {
    /// Repeat a given Chain pseudo-endlessly.
    ///
    /// In reality this adaptpor repeats the underlying elements until `usize::MAX` is reached.
    pub fn new(signal: G) -> Self {
        Repeat(signal)
    }
}

impl<G> Signal<usize> for Repeat<G>
where
    G: Chain,
{
    type Output = G::Output;
    fn eval(&self, input: usize) -> Self::Output {
        self.0.eval(input % self.0.len())
    }
}

impl<G> Chain for Repeat<G>
where
    G: Chain,
{
    fn len(&self) -> usize {
        usize::MAX
    }
}

impl<G> ConstChain<{ usize::MAX }> for Repeat<G> where G: Chain {}

/// Signal adaptor which repeats a fixed amount of first elements.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Wrap<G> {
    inner: G,
    n: usize,
}

impl<G> Wrap<G> {
    /// Wrap the first `n` elements to the end.
    pub fn new(signal: G, n: usize) -> Self {
        Wrap { inner: signal, n }
    }
}

impl<G> Signal<usize> for Wrap<G>
where
    G: Chain,
{
    type Output = G::Output;
    fn eval(&self, input: usize) -> Self::Output {
        self.inner.eval(input % self.inner.len())
    }
}

impl<G> Chain for Wrap<G>
where
    G: Chain,
{
    fn len(&self) -> usize {
        self.inner.len() + self.n
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::easing::Identity;

    #[test]
    fn input_transform() {
        let identity = Identity {};
        let transformed = TransformInput::new(identity, 0.0, 2.0);
        assert_f64_near!(transformed.eval(1.0), 2.0);
        let results = [0.0, 1.0, 2.0];
        // try to extract
        let extractor = transformed.extract([0.0, 0.5, 1.0]);
        for (val, res) in extractor.zip(results.iter()) {
            assert_f64_near!(val, res);
        }
        // try to take - should be the same as before as the domain should have changed accordingly
        let transformed = TransformInput::new(identity, 0.0, 2.0);
        for (val, res) in transformed
            .take(results.len())
            .zip(<Identity as Curve<f64>>::take(identity, results.len()))
        {
            assert_f64_near!(val, res);
        }
    }

    #[test]
    fn slice() {
        let identity = Identity {};
        let slice = Slice::new(identity, 10.0..100.0);
        let results = [10.0, 100.0];
        assert_f64_near!(slice.eval(0.0), 10.0);
        assert_f64_near!(slice.eval(1.0), 100.0);
        for (val, res) in slice.take(results.len()).zip(results.iter()) {
            assert_f64_near!(val, res);
        }
    }
}
