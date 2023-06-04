use num_traits::real::Real;
use num_traits::FromPrimitive;

use core::iter::FusedIterator;
use core::ops::RangeBounds;

use super::Equidistant;
use super::{Clamp, Composite, Repeat, Slice, Stack};

/// Trait which symbolises the generation or copying of an element.
///
/// This trait is fairly similar to core::ops::Index, however it does not retrurn a reference but
/// the element itself. When a struct implements an Index, it usually should be able to implement this trait as well.
/// The other way around does not have to be the case.
pub trait Generator<Input> {
    /// The element outputted
    type Output;
    /// Method to generate the element at the given input
    fn gen(&self, input: Input) -> Self::Output;
    /// Helper function if one wants to extract values from the interpolation.
    ///
    /// It takes an iterator of items which are inputed into the [`gen()`] method
    /// and returns an iterator of the corresponding outputs.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "linear", doc = "```rust")]
    #[cfg_attr(not(feature = "linear"), doc = "```ignore")]
    /// # use enterpolation::{linear::{Linear, LinearError}, Generator};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), LinearError> {
    /// let linear = Linear::builder()
    ///                 .elements([0.0,3.0])
    ///                 .knots([0.0,1.0])
    ///                 .build()?;
    /// let samples = [0.0,0.2,0.4,0.5,0.55,1.0];    // take these samples
    /// let expected = [0.0,0.6,1.2,1.5,1.65,3.0];
    /// for (value, result) in linear.extract(samples).zip(expected) {
    ///     assert_f64_near!(value, result);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`gen()`]: Self::gen()
    fn extract<I, J>(self, iterator: I) -> Extract<Self, J>
    where
        Self: Sized,
        I: IntoIterator<IntoIter = J>,
        J: Iterator<Item = Input>,
    {
        Extract {
            generator: self,
            iterator: iterator.into_iter(),
        }
    }
    /// Stack two generators together
    ///
    /// That is for two generators with output `T` and `R` the created generators output will be `(T,R)`.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "linear", doc = "```rust")]
    #[cfg_attr(not(feature = "linear"), doc = "```ignore")]
    /// # use enterpolation::{linear::{Linear, LinearError}, Generator};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), LinearError> {
    /// let elements = [1.0,5.0,3.0];
    /// let weights = [1.0,3.0,2.0];
    /// // We assume elements and weights to be huge, such that zipping and collecting them is not viable.
    /// let linear = Linear::builder()
    ///                 .elements_with_weights(elements.stack(weights))
    ///                 .knots([0.0,1.0,2.0])
    ///                 .build()?;
    /// assert_f64_near!(linear.gen(0.5), 4.0);
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    fn stack<G>(self, gen: G) -> Stack<Self, G>
    where
        Self: Sized,
    {
        Stack::new(self, gen)
    }
    /// Takes two generators and creates a new generator pipelining both generators.
    ///
    /// [`composite()`] will return a new generator which will first generate values from the original input
    /// and then use these values as input for the second generator.
    ///
    /// In other words, it is the composite of two functions.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "bezier", doc = "```rust")]
    #[cfg_attr(not(feature = "bezier"), doc = "```ignore")]
    /// # use enterpolation::{bezier::{Bezier, BezierError}, easing::{FuncEase, smoothstep}, Generator};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() {
    /// let elements = [-3.0,-2.0,2.0,3.0]; // In reality these would be 2D points etc.
    /// let curve = Bezier::builder()
    ///                 .elements(elements)
    ///                 .normalized::<f64>()
    ///                 .constant::<4>()
    ///                 .build().expect("hardcoded");
    /// // we want to change the velocity of our point transversing the curve
    /// let smoothing = FuncEase::new(smoothstep);
    /// let samples = [0.1,0.25];
    /// let corrected_samples : Vec<_> = smoothing.sample(samples).collect();
    /// let results : Vec<_> = curve.sample(corrected_samples).collect();
    ///
    /// let smoother_animation = smoothing.composite(curve);
    /// assert_f64_near!(smoother_animation.gen(0.1), results[0]);
    /// assert_f64_near!(smoother_animation.gen(0.25), results[1]);
    /// # }
    /// ```
    ///
    /// [`composite()`]: Self::composite()
    fn composite<G>(self, gen: G) -> Composite<Self, G>
    where
        Self: Sized,
    {
        Composite::new(self, gen)
    }
    /// Get a reference of the generator.
    ///
    /// This is useful if one wants to add an adaptor without consuming the original.
    fn by_ref(&self) -> &Self {
        self
    }
    /// Helper function if one wants to sample values from the interpolation.
    ///
    /// It takes an iterator of items which are inputed into the [`gen()`] method
    /// and returns an iterator of the corresponding outputs.
    ///
    /// This acts the same as `generator.by_ref().extract()`.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "linear", doc = "```rust")]
    #[cfg_attr(not(feature = "linear"), doc = "```ignore")]
    /// # use enterpolation::{linear::{Linear, LinearError}, Generator};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), LinearError> {
    /// let linear = Linear::builder()
    ///                 .elements([0.0,3.0])
    ///                 .knots([0.0,1.0])
    ///                 .build()?;
    /// let samples = [0.0,0.2,0.4,0.5,0.55,1.0];    // take these samples
    /// let expected = [0.0,0.6,1.2,1.5,1.65,3.0];
    /// for (value, result) in linear.sample(samples).zip(expected) {
    ///     assert_f64_near!(value, result);
    /// }
    /// // we can still use linear here as it was not consumed!
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`gen()`]: Self::gen()
    fn sample<I, J>(&self, iterator: I) -> Extract<&Self, J>
    where
        Self: Sized,
        I: IntoIterator<IntoIter = J>,
        J: Iterator<Item = Input>,
    {
        self.extract(iterator)
    }
}

// Make references of generators also generators
impl<G: Generator<I> + ?Sized, I> Generator<I> for &G {
    type Output = G::Output;
    fn gen(&self, input: I) -> Self::Output {
        (**self).gen(input)
    }
}

/// Specialized [`Generator`] which takes a real number as input.
///
/// [`Generator`]: Generator
pub trait Curve<R>: Generator<R>
where
    R: Real,
{
    /// The domain in which the curve uses interpolation.
    ///
    /// Not all Curves may extrapolate in a safe way.
    fn domain(&self) -> [R; 2];
    /// Takes equidistant samples of the curve.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "linear", doc = "```rust")]
    #[cfg_attr(not(feature = "linear"), doc = "```ignore")]
    /// # use enterpolation::{linear::{Linear, LinearError}, Curve};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), LinearError> {
    /// let linear = Linear::builder()
    ///                 .elements([0.0,5.0,3.0])
    ///                 .knots([0.0,1.0,2.0])
    ///                 .build()?;
    /// let results = [0.0,1.0,2.0,3.0,4.0,5.0,4.6,4.2,3.8,3.4,3.0];    // take 11 samples
    /// for (value,result) in linear.take(results.len()).zip(results.iter().copied()){
    ///     assert_f64_near!(value, result);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if given size of samples is 0 or if `samples - 1` can not be converted to the type `R`.
    fn take(self, samples: usize) -> Take<Self, R>
    where
        Self: Sized,
        R: FromPrimitive,
    {
        let [start, end] = self.domain();
        Take(self.extract(Stepper::new(samples, start, end)))
    }
    /// Take a slice of a curve.
    ///
    /// A slice of a curve maps its domain onto the given range.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "linear", doc = "```rust")]
    #[cfg_attr(not(feature = "linear"), doc = "```ignore")]
    /// # use enterpolation::{linear::{Linear, LinearError}, Curve};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), LinearError> {
    /// let linear = Linear::builder()
    ///                 .elements([0.0,5.0,3.0])
    ///                 .knots([0.0,1.0,2.0])
    ///                 .build()?;
    /// let sliced_linear = linear.slice(0.5..1.5);
    /// let results = [2.5,5.0,4.0];
    /// for (value,result) in sliced_linear.take(results.len()).zip(results.iter().copied()){
    ///     assert_f64_near!(value, result);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    fn slice<B>(self, bounds: B) -> Slice<Self, R>
    where
        Self: Sized,
        B: RangeBounds<R>,
    {
        Slice::new(self, bounds)
    }
    /// Clamp the input of a curve to its domain.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "linear", doc = "```rust")]
    #[cfg_attr(not(feature = "linear"), doc = "```ignore")]
    /// # use enterpolation::{linear::{Linear, LinearError}, Generator, Curve};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), LinearError> {
    /// let linear = Linear::builder()
    ///                 .elements([0.0,3.0])
    ///                 .knots([0.0,1.0])
    ///                 .build()?
    ///                 .clamp();
    /// let expected = [[-1.0,0.0],[0.0,0.0],[0.5,1.5],[1.0,3.0],[2.0,3.0]];
    /// for [input,result] in expected {
    ///     assert_f64_near!(linear.gen(input), result);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    fn clamp(self) -> Clamp<Self>
    where
        Self: Sized,
    {
        Clamp::new(self)
    }
}

//Make references of curves also curves
impl<C: Curve<R> + ?Sized, R> Curve<R> for &C
where
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        (**self).domain()
    }
}

/// Specialized [`Generator`] with input of type `usize`.
///
/// All `DiscreteGenerator` must return valid values
/// when using inputs less than the value returned by their [`len()`] method.
///
/// [`Generator`]: Generator
/// [`len()`]: DiscreteGenerator::len()
pub trait DiscreteGenerator: Generator<usize> {
    /// Returns the minimum amount of elements the generator can create.
    ///
    /// The generator has to guarantee that every usize number
    /// lower than the returned number has to create a valid element.
    fn len(&self) -> usize;
    /// Returns the first element of the generator, or `None` if it is empty.
    fn first(&self) -> Option<Self::Output> {
        if self.is_empty() {
            return None;
        }
        Some(self.gen(0))
    }
    /// Returns the last element of the generator, or `None` if it is empty.
    fn last(&self) -> Option<Self::Output> {
        if self.is_empty() {
            return None;
        }
        Some(self.gen(self.len() - 1))
    }
    /// Returns `true` if the generator does not generate any elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Convert generator to an iterator which steps through all generatable values.
    fn into_iter(self) -> IntoIter<Self>
    where
        Self: Sized,
    {
        IntoIter::new(self)
    }
    /// Create iterator which steps through all generatable values.
    fn iter(&self) -> IntoIter<&Self> {
        IntoIter::new(self)
    }
    /// Transfrom generator to one which repeats its elements.
    fn repeat(self) -> Repeat<Self>
    where
        Self: Sized,
    {
        Repeat::new(self)
    }
}

// Make references of DiscreteGenerator also DiscreteGenerator
impl<G: DiscreteGenerator + ?Sized> DiscreteGenerator for &G {
    fn len(&self) -> usize {
        (**self).len()
    }
}

/// Trait for [`DiscreteGenerator`] where its length is knwon at compile-time.
///
/// [`DiscreteGenerator`]: DiscreteGenerator
pub trait ConstDiscreteGenerator<const N: usize>: DiscreteGenerator {
    /// Collect all elements generated into an array.
    ///
    /// This function may be useful if one wants to save intermediate steps instead of generating
    /// and caclulating it.
    ///
    /// If you want to transfrom a `DiscreteGenerator` to a collection,
    /// you may use `.iter().collect()` instead.
    fn to_array(&self) -> [Self::Output; N]
    where
        Self::Output: Copy + Default,
    {
        let mut arr = [Default::default(); N];
        for (i, val) in arr.iter_mut().enumerate().take(N) {
            *val = self.gen(i);
        }
        arr
    }
}

//Make references of DiscreteGenerator also DiscreteGenerator
impl<G: ConstDiscreteGenerator<N> + ?Sized, const N: usize> ConstDiscreteGenerator<N> for &G {}

/// Iterator constructed by the `into_iter` and 'iter' method of generators.
#[derive(Debug, Clone, PartialEq)] // Iterators shouldn't be Copy -- see #27186
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IntoIter<G> {
    gen: G,
    front: usize,
    back: usize,
}

impl<G> IntoIter<G>
where
    G: DiscreteGenerator,
{
    pub fn new(gen: G) -> Self {
        IntoIter {
            front: 0,
            back: gen.len(),
            gen,
        }
    }
}

impl<G> Iterator for IntoIter<G>
where
    G: DiscreteGenerator,
{
    type Item = G::Output;
    fn next(&mut self) -> Option<Self::Item> {
        if self.front < self.back {
            let res = self.gen.gen(self.front);
            self.front += 1;
            return Some(res);
        }
        None
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.back - self.front;
        (len, Some(len))
    }
    fn count(self) -> usize {
        self.back - self.front
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.back - self.front {
            return None;
        }
        self.front += n;
        self.next()
    }
}

impl<G> FusedIterator for IntoIter<G> where G: DiscreteGenerator {}

impl<G> ExactSizeIterator for IntoIter<G> where G: DiscreteGenerator {}

impl<G> DoubleEndedIterator for IntoIter<G>
where
    G: DiscreteGenerator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.front < self.back {
            let res = self.gen.gen(self.back);
            self.back -= 1;
            return Some(res);
        }
        None
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if n >= self.back - self.front {
            return None;
        }
        self.back -= n;
        self.next_back()
    }
}

/// Iterator adaptor.
///
/// Maps the items of the iterator to the output of the curve.
///
/// This struct is created by the [`extract()`] method on [`Generator`]. See its documentation for more.
///
/// [`extract()`]: crate::Generator::extract()
/// [`Generator`]: crate::Generator
#[derive(Debug, Clone, PartialEq)] // Iterators shouldn't be Copy -- see #27186
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Extract<G, I> {
    generator: G,
    iterator: I,
}

impl<G, I> Iterator for Extract<G, I>
where
    G: Generator<I::Item>,
    I: Iterator,
{
    type Item = G::Output;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generator.gen(self.iterator.next()?))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
    fn count(self) -> usize {
        self.iterator.count()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.generator.gen(self.iterator.nth(n)?))
    }
}

impl<G, I> FusedIterator for Extract<G, I>
where
    G: Generator<I::Item>,
    I: FusedIterator,
{
}

impl<G, I> ExactSizeIterator for Extract<G, I>
where
    G: Generator<I::Item>,
    I: ExactSizeIterator,
{
}

impl<G, I> DoubleEndedIterator for Extract<G, I>
where
    G: Generator<I::Item>,
    I: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(self.generator.gen(self.iterator.next_back()?))
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.generator.gen(self.iterator.nth_back(n)?))
    }
}

/// Newtype Take to encapsulate implementation details of the curve method take
#[derive(Debug, Clone, PartialEq)] // Iterators shouldn't be Copy -- see #27186
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Take<C, R>(Extract<C, Stepper<R>>)
where
    R: Real;

impl<C, R> Iterator for Take<C, R>
where
    C: Curve<R>,
    R: Real + FromPrimitive,
{
    type Item = C::Output;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize {
        self.0.count()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }
}

impl<C, R> FusedIterator for Take<C, R>
where
    C: Curve<R>,
    R: Real + FromPrimitive,
{
}

impl<C, R> ExactSizeIterator for Take<C, R>
where
    C: Curve<R>,
    R: Real + FromPrimitive,
{
}

impl<C, R> DoubleEndedIterator for Take<C, R>
where
    C: Curve<R>,
    R: Real + FromPrimitive,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

/// Stepper is an iterator which increments its number.
///
/// Stepper can be seen as a [`Range`] with variable step size.
///
/// [`Range`]: core::ops::Range
#[derive(Debug, Clone, PartialEq)] // Iterators shouldn't be Copy -- see #27186
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Stepper<R: Real = f64>(IntoIter<Equidistant<R>>);

impl<R> Stepper<R>
where
    R: Real + FromPrimitive,
{
    /// Creates a new Stepper stepping from 0 to 1
    /// Also the given steps are not allowed to be less than 1
    ///
    /// #Panics
    ///
    /// Panics if the given steps are 0 and if `steps -1` can not be transformed into R.
    pub fn normalized(steps: usize) -> Self {
        Stepper(Equidistant::normalized(steps).into_iter())
    }

    /// Creates a new Stepper stepping from `start` to `end`
    /// Also the given steps are not allowed to be less than 1
    ///
    /// #Panics
    ///
    /// Panics if the given steps are 0 and if `steps -1` can not be transformed into R.
    pub fn new(steps: usize, start: R, end: R) -> Self {
        Stepper(Equidistant::new(steps, start, end).into_iter())
    }
}

impl<R> Iterator for Stepper<R>
where
    R: Real + FromPrimitive,
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
    fn count(self) -> usize {
        self.0.count()
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n)
    }
}

impl<R> FusedIterator for Stepper<R> where R: Real + FromPrimitive {}

impl<R> ExactSizeIterator for Stepper<R> where R: Real + FromPrimitive {}

impl<R> DoubleEndedIterator for Stepper<R>
where
    R: Real + FromPrimitive,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stepper() {
        let mut stepper = Stepper::normalized(11);
        let res = [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        for i in 0..=10 {
            let val = stepper.next().unwrap();
            assert_f64_near!(val, res[i]);
        }

        let mut stepper = Stepper::new(5, 3.0, 5.0);
        let res = [3.0, 3.5, 4.0, 4.5, 5.0];
        for i in 0..5 {
            let val = stepper.next().unwrap();
            assert_f64_near!(val, res[i]);
        }
    }
}
