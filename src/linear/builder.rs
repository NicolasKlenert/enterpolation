//! Builder module for linear interpolations.
//!
//! Each interpolation has it's own builder module, which accumalates all methods to create their interpolation.

//TODO: EXAMPLE

use core::ops::Mul;
use core::marker::PhantomData;
use num_traits::real::Real;
use num_traits::FromPrimitive;
use num_traits::identities::Zero;
use crate::{Generator, DiscreteGenerator, SortedGenerator, Sorted, Equidistant, Merge, Identity};
use crate::weights::{Weighted, Weights, IntoWeight};
use crate::builder::{WithWeight,WithoutWeight, Type, Unknown, Unsorted};
use super::Linear;
use super::error::LinearError;

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
pub struct LinearBuilder<K,E,F,W> {
    knots: K,
    elements: E,
    easing: F,
    _phantom: PhantomData<*const W>,
}

impl Default for LinearBuilder<Unknown, Unknown, Identity, Unknown> {
    fn default() -> Self {
        LinearBuilder::new()
    }
}

impl LinearBuilder<Unknown, Unknown, Identity, Unknown> {
    /// Create a new linear interpolation builder.
    pub const fn new() -> Self {
        LinearBuilder {
            knots: Unknown,
            elements: Unknown,
            easing: Identity::new(),
            _phantom: PhantomData,
        }
    }
}

impl<F> LinearBuilder<Unknown, Unknown, F, Unknown> {
    /// Set the elements of the linear interpolation.
    pub fn elements<E>(self, elements: E) -> LinearBuilder<Unknown, E, F, WithoutWeight>
    where E: DiscreteGenerator,
    {
        LinearBuilder {
            knots: self.knots,
            elements,
            easing: self.easing,
            _phantom: PhantomData,
        }
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
    /// ```rust
    /// # use enterpolation::{linear::{Linear, LinearError}, Generator, Curve};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), LinearError> {
    /// let linear = Linear::builder()
    ///                 .elements_with_weights([(1.0,1.0),(2.0,4.0),(3.0,0.0)])
    ///                 .equidistant::<f64>()
    ///                 .normalized()
    ///                 .build()?;
    /// let results = [1.0,1.8,2.0,2.75,f64::INFINITY];
    /// for (value,result) in linear.take(5).zip(results.iter().copied()){
    ///     assert_f64_near!(value, result);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn elements_with_weights<G>(self, gen: G) -> LinearBuilder<Unknown, Weights<G>, F, WithWeight>
    where
        G: DiscreteGenerator,
        G::Output: IntoWeight,
        <G::Output as IntoWeight>::Element:
            Mul<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
        <G::Output as IntoWeight>::Weight: Zero + Copy,
    {
        LinearBuilder {
            knots: self.knots,
            elements: Weights::new(gen),
            easing: self.easing,
            _phantom: PhantomData,
        }
    }
}

impl<E,F,W> LinearBuilder<Unknown, E, F, W>
{
    /// Set the knots of the interpolation.
    ///
    /// The amount of knots must be equal to the amount of elements.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using `equidistant()` instead.
    pub fn knots<K>(self, knots: K) -> LinearBuilder<Unsorted<K>,E, F, W>
    where
        E: DiscreteGenerator,
        K: DiscreteGenerator,
        K::Output: PartialOrd
    {
        LinearBuilder {
            knots: Unsorted::new(knots),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }

    /// Build an interpolation with equidistant knots.
    pub fn equidistant<R>(self) -> LinearBuilder<Type<R>,E,F,W>{
        LinearBuilder {
            knots: Type::new(),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }
}

impl<R,E,F,W> LinearBuilder<Type<R>,E,F,W>
where
    E: DiscreteGenerator,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(self, start: R, end: R) -> LinearBuilder<Equidistant<R>,E,F,W>{
        LinearBuilder {
            knots: Equidistant::new(self.elements.len(), start, end),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(self) -> LinearBuilder<Equidistant<R>,E,F,W>{
        LinearBuilder {
            knots: Equidistant::normalized(self.elements.len()),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots
    pub fn distance(self, start: R, step: R) -> LinearBuilder<Equidistant<R>,E,F,W>{
        LinearBuilder {
            knots: Equidistant::step(self.elements.len(), start, step),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }
}

impl<K,E,F,W> LinearBuilder<K,E,F,W>
where
    K: SortedGenerator
{
    /// Set another easing function.
    ///
    /// This interpolation uses a factor to merge elements together.
    /// The dynamic how to merge these two elements together can be changed by easing the factor itself
    /// before merging.
    ///
    /// # Examples
    ///
    /// See the plateau example for more information.
    pub fn easing<FF>(self, easing: FF) -> LinearBuilder<K,E,FF,W>{
        LinearBuilder {
            knots: self.knots,
            elements: self.elements,
            easing,
            _phantom: self._phantom,
        }
    }
}

impl<K,E,F> LinearBuilder<K,E,F,WithoutWeight>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    E::Output: Merge<K::Output>,
    K::Output: Real
{
    /// Build a linear interpolation.
    pub fn build(self) -> Result<Linear<K,E,F>,LinearError>{
        // knots are sorted
        Linear::new(self.elements, self.knots, self.easing)
    }
}

impl<K,G,F> LinearBuilder<K,Weights<G>,F,WithWeight>
where
    K: SortedGenerator,
    K::Output: Real + Copy,
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    <Weights<G> as Generator<usize>>::Output: Merge<K::Output>,
{
    /// Build a weighted linear interpolation.
    pub fn build(self) -> Result<Weighted<Linear<K,Weights<G>,F>>, LinearError>
    {
        // knots are sorted
        Ok(Weighted::new(Linear::new(self.elements, self.knots, self.easing)?))
    }
}

impl<K,E,F> LinearBuilder<Unsorted<K>,E,F,WithoutWeight>
where
    E: DiscreteGenerator,
    K: DiscreteGenerator,
    E::Output: Merge<K::Output>,
    K::Output: Real
{
    /// Build a linear interpolation.
    pub fn build(self) -> Result<Linear<Sorted<K>,E,F>,LinearError>{
        Linear::new(self.elements, Sorted::new(self.knots.0)?, self.easing)
    }
}

impl<K,G,F> LinearBuilder<Unsorted<K>,Weights<G>,F,WithWeight>
where
    K: DiscreteGenerator,
    K::Output: Real + Copy,
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    <Weights<G> as Generator<usize>>::Output: Merge<K::Output>,
{
    /// Build a weighted linear interpolation.
    pub fn build(self) -> Result<Weighted<Linear<Sorted<K>,Weights<G>,F>>, LinearError>
    {
        Ok(Weighted::new(Linear::new(self.elements, Sorted::new(self.knots.0)?, self.easing)?))
    }
}

#[cfg(test)]
mod test {
    use super::LinearBuilder;
    // Homogeneous for creating Homogeneous, Generator for using .stack(
    use crate::{weights::Homogeneous, Generator};
    #[test]
    fn building_weights() {
        LinearBuilder::new()
            .elements_with_weights([(1.0,1.0),(2.0,2.0),(3.0,0.0)])
            .equidistant::<f64>()
            .normalized()
            .build().unwrap();
        LinearBuilder::new()
            .elements_with_weights([1.0,2.0,3.0].stack([1.0,2.0,0.0]))
            .equidistant::<f64>()
            .normalized()
            .build().unwrap();
        LinearBuilder::new()
            .elements_with_weights([
                Homogeneous::new(1.0),
                Homogeneous::weighted_unchecked(2.0, 2.0),
                Homogeneous::infinity(3.0)])
            .knots([1.0,2.0,3.0])
            .build().unwrap();
        LinearBuilder::new()
            .elements([0.1,0.2,0.3])
            .equidistant::<f64>()
            .normalized()
            .build().unwrap();
        assert!(LinearBuilder::new().elements::<[f64;0]>([]).knots::<[f64;0]>([]).build().is_err());
        assert!(LinearBuilder::new().elements([1.0]).knots([1.0]).build().is_err());
        assert!(LinearBuilder::new().elements([1.0,2.0]).knots([1.0,2.0,3.0]).build().is_err());
    }
}
