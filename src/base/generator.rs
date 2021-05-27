use core::ops::Range;
use num_traits::real::Real;
use num_traits::FromPrimitive;

use super::Equidistant;

/// Trait which symbolises the generation or copying of an element.
///
/// This trait is fairly similar to core::ops::Index, however it does not retrurn a reference but
/// the element itself.
pub trait Generator<Input> {
    /// The element outputted
    type Output;
    /// Function to generate the element
    fn get(&self, input: Input) -> Self::Output;
    /// Helper function if one wants to sample the interpolation.
    /// It takes an iterator which yields items which are inputted into the `get` function
    /// and returns the output of the interpolation.
    fn extract<I>(self, iterator: I) -> Extract<Self, I>
    where
        Self: Sized,
        I: Iterator<Item = Input>
    {
        Extract {
            generator: self,
            iterator,
        }
    }
    /// Get a reference of the generator.
    /// This is useful if one wants to add an adapter without consuming the original.
    fn by_ref(&self) -> &Self {
        self
    }
}

//Make references of generators also generators
impl<G: Generator<I> + ?Sized,I> Generator<I> for &G {
    type Output = G::Output;
    fn get(&self, input: I) -> Self::Output {
        (**self).get(input)
    }
}

/// Trait for all Interpolations.
///
/// Interpolations are nothing else then Generators which are guaranteeing that
/// the generation of elements inbetween some specific points is smooth.
pub trait Interpolation<Input> : Generator<Input>
{}

//Make references of interpolations also interpolations
impl<I: Interpolation<Input> + ?Sized,Input> Interpolation<Input> for &I {}

/// Curve is a specialized Interpolation which takes a real number as input
pub trait Curve<R> : Interpolation<R>
where R: Real
{
    /// The domain in which the curve uses interpolation. Not all Curves may extrapolate in a safe way.
    fn domain(&self) -> [R; 2];
    /// Takes equidistant samples of the curve (with 0.0 and 1.0 inclusive).
    fn take(&self, samples: usize) -> Take<&Self, R>
    where R: FromPrimitive
    {
        Take(self.extract(Stepper::new(samples)))
    }
}

//Make references of interpolations also interpolations
impl<C: Curve<R> + ?Sized,R> Curve<R> for &C
where R: Real
{
    fn domain(&self) -> [R; 2] {
        (**self).domain()
    }
}

/// FiniteGenerator are generators which only guarantee creation of elements if the input is lower than their length.
pub trait FiniteGenerator : Generator<usize> {
    /// Returns the minimum amount of elements the generator can create.
    ///
    /// The generator has to guarantee that every usize number
    /// lower than the returned number has to create a valid element.
    fn len(&self) -> usize;
    /// Returns the first element of the generator, or None if it is empty.
    fn first(&self) -> Option<Self::Output> {
        if self.is_empty(){
            return None;
        }
        Some(self.get(0))
    }
    /// Returns the last element of the generator, or None if it is empty.
    fn last(&self) -> Option<Self::Output> {
        if self.is_empty(){
            return None;
        }
        Some(self.get(self.len() - 1))
    }
    /// Returns true if the generator does not generate any elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Iterator adaptor, which transforms an iterator with InterScalar items to an iterator with the correspondending values of the interpolation
pub struct Extract<G, I> {
    generator: G,
    iterator: I,
}

impl<G, I> Iterator for Extract<G, I>
where
    G: Generator<I::Item>,
    I: Iterator
{
    type Item = G::Output;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generator.get(self.iterator.next()?))
    }
}

/// Newtype Take to encapsulate implementation details of the curve method take
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
}

/// Newtype Steper to encapsulate implementation details.
/// Stepper is an Iterator which steps from 0.0 to 1.0 in a specific amount of constant steps.
pub struct Stepper<R: Real = f64>(Extract<Equidistant<R>,Range<usize>>);

impl<R> Stepper<R>
where
    R: Real + FromPrimitive,
{
    /// Creates a new Stepper stepping from 0 to 1
    /// The given generic real number has to be able to be created from usize
    /// Also the given steps are not allowed to be less than 1
    pub fn new(steps: usize) -> Self {
        Stepper(Equidistant::new(steps).extract(0..steps))
    }
}

impl<R> Iterator for Stepper<R>
where R: Real + FromPrimitive
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}