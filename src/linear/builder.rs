//! Builder module for linear interpolations.
//!
//! Each interpolation has it's own builder module, which accumalates all methods to create their interpolation.

//TODO: EXAMPLE

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use num_traits::real::Real;
use num_traits::FromPrimitive;
use num_traits::identities::Zero;
use crate::{Generator, DiscreteGenerator, SortedGenerator, Sorted, Equidistant, Homogeneous, Weighted, Weights, IntoWeight};
use super::Linear;
use super::error::{LinearError, ToFewElements, KnotElementInequality};

//TODO: add unchecked versions

/// Struct indicator to mark if we use weights
#[derive(Debug, Copy, Clone)]
pub struct WithWeight<T>(T);

/// Struct indicator to mark information not yet given.
#[derive(Debug, Copy, Clone)]
pub struct Unknown;

/// Struct indicator to mark the wish of using equidistant knots.
#[derive(Debug, Copy, Clone)]
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
    ///
    /// Weights of `Zero` can achieve unwanted results as their corresponding elements are considered
    /// to be at infinity.
    /// In this case the interpolation may generate NaN, infinite or even panic as elements
    /// are divided by `Zero`.
    ///
    /// If you want to work with points at infinity,
    /// you may want to use homogeneous data itself without this wrapping mechanism.
    ///
    /// # Examples
    ///
    /// ```
    /// use enterpolation::{linear::Linear, Generator, Curve};
    /// let linear = Linear::builder()
    ///                 .elements_with_weights([(1.0,1.0),(2.0,4.0),(3.0,0.0)]).unwrap()
    ///                 .equidistant::<f64>()
    ///                 .build();
    /// let results = [1.0,1.8,2.0,2.75,f64::INFINITY];
    /// for (value,result) in linear.take(5).zip(results.iter().copied()){
    ///     assert_eq!(value, result);
    /// }
    /// ```
    pub fn elements_with_weights<G>(self, gen: G)
        -> Result<LinearBuilder<Unknown, WithWeight<Weights<G>>>,ToFewElements>
    where
        G: DiscreteGenerator,
        G::Output: IntoWeight,
        <G::Output as IntoWeight>::Element:
            Mul<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
        <G::Output as IntoWeight>::Weight: Zero + Copy,
    {
        if gen.len() < 2 {
            return Err(ToFewElements::new(gen.len()));
        }
        Ok(LinearBuilder {
            knots: self.knots,
            elements: WithWeight(Weights::new(gen)),
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

impl<K,G> LinearBuilder<K,WithWeight<Weights<G>>>
where
    K: SortedGenerator,
    K::Output: Real + Copy,
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    <Weights<G> as Generator<usize>>::Output:
        Add<Output = <Weights<G> as Generator<usize>>::Output> +
        Mul<K::Output, Output = <Weights<G> as Generator<usize>>::Output> +
        Copy
{
    /// Build a weighted linear interpolation.
    pub fn build(self) -> Weighted<Linear<K,Weights<G>>>
    {
        Weighted::new(Linear::new(self.elements.0, self.knots).unwrap())
    }
}

impl<R,G> LinearBuilder<Output<R>,WithWeight<Weights<G>>>
where
    R: Real + Copy + FromPrimitive,
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    <Weights<G> as Generator<usize>>::Output:
        Add<Output = <Weights<G> as Generator<usize>>::Output> +
        Mul<R, Output = <Weights<G> as Generator<usize>>::Output> +
        Copy,
{
    /// Build a weighted linear interpolation from a vector of elements and equidistant knots in [0.0,1.0].
    pub fn build(self) -> Weighted<Linear<Equidistant<R>,Weights<G>>> {
        let len = self.elements.0.len();
        let knots = Equidistant::normalized(len);
        Weighted::new(Linear::new(self.elements.0, knots).unwrap())
    }
    /// Build a weighted linear interpolation from a vector of elements and equidistant knots in the specified domain.
    pub fn build_with_domain(self, start:R, end: R) -> Weighted<Linear<Equidistant<R>,Weights<G>>> {
        let len = self.elements.0.len();
        let knots = Equidistant::new(start, end, len);
        Weighted::new(Linear::new(self.elements.0, knots).unwrap())
    }
}

// possible variations:
// elements (1) or elements_with_weights (3)
// knots (1) or equidistant (1) [try to create a const building of equidistant]

#[cfg(test)]
mod test {
    use super::LinearBuilder;
    // Homogeneous for creating Homogeneous, Generator for using .stack()
    use crate::{Homogeneous, Generator};
    #[test]
    fn elements_with_weights() {
        LinearBuilder::new()
            .elements_with_weights([(1.0,1.0),(2.0,2.0),(3.0,0.0)]).unwrap()
            .equidistant::<f64>()
            .build();
        LinearBuilder::new()
            .elements_with_weights([1.0,2.0,3.0].stack([1.0,2.0,0.0])).unwrap()
            .equidistant::<f64>()
            .build();
        LinearBuilder::new()
            .elements_with_weights([
                Homogeneous::new(1.0),
                Homogeneous::weighted_unchecked(2.0, 2.0),
                Homogeneous::infinity(3.0)]).unwrap()
            .equidistant::<f64>()
            .build();
    }
}
