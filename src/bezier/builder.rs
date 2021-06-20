//! Builder module for bezier interpolations.
//!
//! Each interpolation has it's own builder module, which accumalates all methods to create their interpolation.

//TODO: EXAMPLE

use core::ops::{Add, Mul, Div};
use core::marker::PhantomData;
use num_traits::real::Real;
use num_traits::identities::Zero;
use crate::{Generator, DiscreteGenerator, ConstDiscreteGenerator, Weighted, Weights,
    IntoWeight, Homogeneous, Space, TransformInput, DynSpace, ConstSpace};
use crate::builder::{WithWeight,Unknown};
use super::Bezier;
use super::error::Empty;
// use super::error::{LinearError, ToFewElements, KnotElementInequality};

//TODO: add const and dyn functin (or only const) to set the space!

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
pub struct BezierBuilder<R,E,S> {
    _input: PhantomData<R>,
    elements: E,
    space: S,
}

impl Default for BezierBuilder<Unknown, Unknown, Unknown> {
    fn default() -> Self {
        BezierBuilder::new()
    }
}

impl BezierBuilder<Unknown, Unknown, Unknown> {
    /// Create a new linear interpolation builder.
    pub const fn new() -> Self {
        BezierBuilder {
            _input: PhantomData,
            elements: Unknown,
            space: Unknown,
        }
    }
}

impl BezierBuilder<Unknown, Unknown, Unknown> {
    /// Set the elements of the linear interpolation.
    pub fn elements<E>(self, elements: E) -> Result<BezierBuilder<Unknown, E, Unknown>, Empty>
    where E: DiscreteGenerator,
    {
        if elements.is_empty() {
            return Err(Empty::new());
        }
        Ok(BezierBuilder {
            _input: self._input,
            space: self.space,
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
        -> Result<BezierBuilder<Unknown, WithWeight<Weights<G>>,Unknown>,Empty>
    where
        G: DiscreteGenerator,
        G::Output: IntoWeight,
        <G::Output as IntoWeight>::Element:
            Mul<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
        <G::Output as IntoWeight>::Weight: Zero + Copy,
    {
        if gen.is_empty() {
            return Err(Empty::new());
        }
        Ok(BezierBuilder {
            _input: self._input,
            space: self.space,
            elements: WithWeight(Weights::new(gen)),
        })
    }
}

impl<E> BezierBuilder<Unknown, E, Unknown>
{
    /// Set the input type used for this interpolation.
    pub fn input<R>(self) -> BezierBuilder<R,E,Unknown>{
        BezierBuilder {
            _input: PhantomData,
            space: self.space,
            elements: self.elements,
        }
    }
}

impl<R,E> BezierBuilder<R,E, Unknown>
where E: DiscreteGenerator
{
    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder to use a vector as workspace,
    /// such you don't need to know the degree of the bezier curve at compile-time,
    /// but every generation of a value an allocation of memory will be necessary.
    pub fn dynamic(self) -> BezierBuilder<R,E,DynSpace<E::Output>>{
        BezierBuilder{
            _input: self._input,
            space: DynSpace::new(self.elements.len()),
            elements: self.elements,
        }
    }

    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder the size of the workspace needed such that no memory allocations are needed
    /// when interpolating.
    pub fn constant<const N: usize>(self) -> BezierBuilder<R,E,ConstSpace<E::Output,N>>
    where E: ConstDiscreteGenerator<N>
    {
        BezierBuilder{
            _input: self._input,
            space: ConstSpace::new(),
            elements: self.elements,
        }
    }

    /// Set the workspace whcih the interpolation uses.
    ///
    /// The workspace has to have a size of the number of elements in the bezier curve.
    ///
    /// If the degree of the bezier curve is known at compile-time, consider using `constant` instead.
    /// Otherwise without std support, one has to set a specific object implementing the `Space` trait.
    pub fn workspace<S>(self, space: S) -> BezierBuilder<R,E,S>
    where S: Space<E::Output>
    {
        //TODO: return error instead of panic
        assert!(space.len() >= self.elements.len());

        BezierBuilder{
            _input: self._input,
            space,
            elements: self.elements,
        }
    }
}

impl<R,E,S> BezierBuilder<R,E,S>
where
    E: DiscreteGenerator,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy,
    S: Space<E::Output>,
    R: Real
{
    /// Build a bezier interpolation.
    pub fn build(self) -> Bezier<R,E,S>{
        Bezier::new_unchecked(self.elements, self.space)
    }

    /// Build a bezier interpolation with the given domain
    pub fn build_with_domain(self, start: R, end: R) -> TransformInput<Bezier<R,E,S>,R,R> {
        TransformInput::normalized_to_domain(Bezier::new_unchecked(self.elements, self.space), start, end)
    }
}

impl<R,G,S> BezierBuilder<R,WithWeight<Weights<G>>,S>
where
    R: Real + Copy,
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    S: Space<Homogeneous<<G::Output as IntoWeight>::Element, <G::Output as IntoWeight>::Weight>>,
    <Weights<G> as Generator<usize>>::Output:
        Add<Output = <Weights<G> as Generator<usize>>::Output> +
        Mul<R, Output = <Weights<G> as Generator<usize>>::Output> +
        Copy,
    <G::Output as IntoWeight>::Element: Div<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
{
    /// Build a weighted bezier interpolation.
    pub fn build(self) -> Weighted<Bezier<R,Weights<G>,S>>
    {
        Weighted::new(Bezier::new_unchecked(self.elements.0, self.space))
    }
    /// Build a weighted bezier interpolation with given domain.
    pub fn build_with_domain(self, start: R, end: R) -> TransformInput<Weighted<Bezier<R,Weights<G>,S>>,R,R> {
        TransformInput::normalized_to_domain(Weighted::new(Bezier::new_unchecked(self.elements.0, self.space)), start, end)
    }
}

// possible variations:
// elements (1) or elements_with_weights (3)

#[cfg(test)]
mod test {
    use super::BezierBuilder;
    // Homogeneous for creating Homogeneous, Generator for using .stack()
    use crate::{Homogeneous, Generator};
    #[test]
    fn elements_with_weights() {

    }
}
