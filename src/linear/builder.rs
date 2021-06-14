//! Builder module for linear interpolations.
//!
//! Each interpolation has it's own builder module, which accumalates all methods to create their interpolation.

//TODO: EXAMPLE

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use num_traits::real::Real;
use num_traits::FromPrimitive;
use num_traits::identities::Zero;
use crate::{DiscreteGenerator, SortedGenerator, Sorted, Equidistant, Homogeneous, Weighted};
use super::Linear;
use super::error::{LinearError, ToFewElements, WeightOfZero, KnotElementInequality};

//TODO: add unchecked versions

/// Struct indicator to mark if we use weights
#[derive(Debug)]
pub struct WithWeight<T>(T);

/// Struct indicator to mark information not yet given.
#[derive(Debug)]
pub struct Unknown;

/// Struct indicator to mark the wish of using equidistant knots.
#[derive(Debug)]
pub struct Output<R = f64>(PhantomData<*const R>);

impl<R> Output<R> {
    pub fn new() -> Self {
        Output(PhantomData)
    }
}

/// Builder for linear interpolation.
///
/// This struct helps create linear interpolations.
/// Usually one creates an instance by using the `builder()` method on the interpolation itself.
///
/// Before building, one has to give information for:
/// - The elements the interpolation should use. Methods like `elements` and `elements_with_weights`
/// exist for that cause.
/// - The knots the interpolation uses. Either by giving them directly with `knots` or by using
/// equidistant knots with `equidistant`.
#[derive(Debug, Clone)]
pub struct LinearBuilder<K,E> {
    knots: K,
    elements: E,
}

impl Default for LinearBuilder<Unknown, Unknown> {
    fn default() -> Self {
        LinearBuilder::new()
    }
}

impl LinearBuilder<Unknown, Unknown> {
    /// Create a new linear interpolation builder.
    pub const fn new() -> Self {
        LinearBuilder {
            knots: Unknown,
            elements: Unknown,
        }
    }
}

impl LinearBuilder<Unknown, Unknown> {
    /// Set the elements of the linear interpolation.
    pub fn elements<E>(self, elements: E) -> Result<LinearBuilder<Unknown, E>, ToFewElements>
    where E: DiscreteGenerator,
    {
        if elements.len() < 2 {
            return Err(ToFewElements::new(elements.len()));
        }
        Ok(LinearBuilder {
            knots: self.knots,
            elements,
        })
    }

    /// Set the elements and their weights for this interpolation.
    pub fn elements_with_weights<I,W,E>(self, iter: I)
        -> Result<LinearBuilder<Unknown, WithWeight<Vec<Homogeneous<E,W>>>>,LinearError>
    where
        I: IntoIterator<Item = (E,W)>,
        W: Zero + Copy,
        E: Mul<W, Output = E>,
    {
        // we can not use iterator style, as a closure does not work nicely with ? syntax.
        let iter = iter.into_iter();
        let mut vec = Vec::with_capacity(iter.size_hint().0);
        for (element, weight) in iter {
            vec.push(Homogeneous::weighted(element,weight).ok_or_else(|| WeightOfZero::new())?);
        }
        if vec.len() < 2 {
            return Err(ToFewElements::new(vec.len()).into());
        }
        Ok(LinearBuilder {
            knots: self.knots,
            elements: WithWeight(vec),
        })
    }

    /// Set the elements and their weights for this interpolation.
    ///
    /// # Performance
    ///
    /// This functions takes only arrays but is such able to store the data necessary for interpolation
    /// into an array. No memory allcations are done therefore.
    pub fn elements_with_weights_array<T,R,const N: usize>(self, arr: [(T,R);N])
            -> Result<LinearBuilder<Unknown, WithWeight<[Homogeneous<T,R>;N]>>, LinearError>
    where
        R: Zero + Default + Copy,
        T: Default + Copy + Mul<R, Output = T>,
    {
        if N < 2 {
            return Err(ToFewElements::new(N).into());
        }
        let mut elements = [Default::default();N];
        for i in 0..N {
            elements[i] = Homogeneous::weighted(arr[i].0,arr[i].1).ok_or_else(|| WeightOfZero::new())?;
        }
        Ok(LinearBuilder {
            knots: self.knots,
            elements: WithWeight(elements),
        })
    }
}

impl<E> LinearBuilder<Unknown, E>
{
    /// Set the knots of the interpolation.
    ///
    /// The amount of knots must be equal to the amount of elements.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using `equidistant()` instead.
    pub fn knots<K>(self, knots: K) -> Result<LinearBuilder<Sorted<K>,E>, LinearError>
    where
        E: DiscreteGenerator,
        K: DiscreteGenerator,
        K::Output: PartialOrd
    {
        if self.elements.len() != knots.len() {
            return Err(KnotElementInequality::new(self.elements.len(), knots.len()).into());
        }
        Ok(LinearBuilder {
            knots: Sorted::new(knots)?,
            elements: self.elements,
        })
    }

    /// Build an interpolation with equidistant knots.
    pub fn equidistant<R>(self) -> LinearBuilder<Output<R>,E>{
        LinearBuilder {
            knots: Output::new(),
            elements: self.elements,
        }
    }
}

impl<K,E> LinearBuilder<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    E::Output: Add<Output = E::Output> + Mul<K::Output, Output = E::Output> + Copy,
    K::Output: Real
{
    /// Build a linear interpolation.
    pub fn build(self) -> Linear<K,E>{
        Linear::new(self.elements, self.knots).unwrap()
    }
}

impl<K,T,R> LinearBuilder<K,WithWeight<Vec<Homogeneous<T,R>>>>
where
    K: SortedGenerator<Output = R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Copy
{
    /// Build a weighted linear interpolation.
    pub fn build(self) -> Weighted<Linear<K,Vec<Homogeneous<T,R>>>,T,R>{
        Weighted::new(Linear::new(self.elements.0, self.knots).unwrap())
    }
}

impl<K,T,R, const N: usize> LinearBuilder<K,WithWeight<[Homogeneous<T,R>;N]>>
where
    K: SortedGenerator<Output = R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Copy
{
    /// Build a weighted linear interpolation with backing arrays.
    pub fn build(self) -> Weighted<Linear<K,[Homogeneous<T,R>;N]>,T,R>{
        Weighted::new(Linear::new(self.elements.0, self.knots).unwrap())
    }
}

impl<R,E> LinearBuilder<Output<R>, E>
where
    E: DiscreteGenerator,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy,
    R: Real + FromPrimitive
{
    /// Build a linear interpolation with equidistant knots with domain [0.0,1.0].
    pub fn build(self) -> Linear<Equidistant<R>,E> {
        let len = self.elements.len();
        Linear::new(self.elements, Equidistant::normalized(len)).unwrap()
    }

    /// Build a linear interpolation with equidistant knots in the specified domain.
    pub fn build_with_domain(self, start:R, end: R) -> Linear<Equidistant<R>,E> {
        let len = self.elements.len();
        Linear::new(self.elements, Equidistant::new(start, end, len)).unwrap()
    }
}

impl<T,R> LinearBuilder<Output<R>,WithWeight<Vec<Homogeneous<T,R>>>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Copy + FromPrimitive
{
    /// Build a weighted linear interpolation from a vector of elements and equidistant knots in [0.0,1.0].
    pub fn build(self) -> Weighted<Linear<Equidistant<R>,Vec<Homogeneous<T,R>>>,T,R> {
        let len = self.elements.0.len();
        let knots = Equidistant::normalized(len);
        Weighted::new(Linear::new(self.elements.0, knots).unwrap())
    }
    /// Build a weighted linear interpolation from a vector of elements and equidistant knots in the specified domain.
    pub fn build_with_domain(self, start:R, end: R) -> Weighted<Linear<Equidistant<R>,Vec<Homogeneous<T,R>>>,T,R> {
        let len = self.elements.0.len();
        let knots = Equidistant::new(start, end, len);
        Weighted::new(Linear::new(self.elements.0, knots).unwrap())
    }
}

impl<T,R, const N: usize> LinearBuilder<Output<R>,WithWeight<[Homogeneous<T,R>;N]>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive + Copy
{
    /// Build a weighted linear interpolation from an array of elements and equidistant knots in [0.0,1.0].
    pub fn build(self) -> Weighted<Linear<Equidistant<R>,[Homogeneous<T,R>;N]>,T,R>{
        let len = self.elements.0.len();
        let knots = Equidistant::normalized(len);
        Weighted::new(Linear::new(self.elements.0,knots).unwrap())
    }
    /// Build a weighted linear interpolation from an array of elements and equidistant knots in the specified domain.
    pub fn build_with_domain(self, start:R, end: R) -> Weighted<Linear<Equidistant<R>,[Homogeneous<T,R>;N]>,T,R> {
        let len = self.elements.0.len();
        let knots = Equidistant::new(start, end, len);
        Weighted::new(Linear::new(self.elements.0, knots).unwrap())
    }
}
