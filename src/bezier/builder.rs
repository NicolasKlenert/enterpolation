//! Builder module for bezier interpolations.
//!
//! Each interpolation has it's own builder module, which accumalates all methods to create their interpolation.

use super::error::BezierError;
use super::Bezier;
use crate::builder::{InputDomain, NormalizedInput, Unknown, WithWeight, WithoutWeight};
use crate::weights::{Homogeneous, IntoWeight, Weighted, Weights};
#[cfg(feature = "std")]
use crate::DynSpace;
use crate::{
    ConstDiscreteGenerator, ConstSpace, DiscreteGenerator, Generator, Space, TransformInput,
};
use core::marker::PhantomData;
use core::ops::{Div, Mul};
use num_traits::identities::Zero;
use num_traits::real::Real;
use topology_traits::Merge;

/// Builder for bezier curves.
///
/// This struct helps create bezier curves.
/// Usually one creates an instance by using the `builder()` method on the interpolation itself.
///
/// Before building, one has to give information for:
/// - which elements to use with [`elements()`] or [`elements_with_weights()`],
/// - the domain of the bezier curve with the standard [`normalized()`] domain or a custom [`domain()`],
/// - the kind of workspace to use with [`dynamic()`], [`constant()`] or [`workspace()`]
///
/// # Examples
///
/// ```rust
/// # use std::error::Error;
/// # use enterpolation::{bezier::{Bezier, BezierError}, Generator, Curve};
/// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
/// #
/// # fn main() -> Result<(), BezierError> {
/// let bez = Bezier::builder()
///     .elements([20.0,100.0,0.0,200.0])
///     .normalized::<f64>()
///     .constant()
///     .build()?;
/// let mut iter = bez.take(5);
/// let expected = [20.0,53.75,65.0,98.75,200.0];
/// for i in 0..=4 {
///     let val = iter.next().unwrap();
///     assert_f64_near!(val, expected[i]);
/// }
/// #
/// #     Ok(())
/// # }
/// ```
///
/// [`elements()`]: BezierBuilder::elements()
/// [`elements_with_weights()`]: BezierBuilder::elements_with_weights()
/// [`normalized()`]: BezierBuilder::normalized()
/// [`domain()`]: BezierBuilder::domain()
/// [`dynamic()`]: BezierBuilder::dynamic()
/// [`constant()`]: BezierBuilder::constant()
/// [`workspace()`]: BezierBuilder::workspace()
#[derive(Debug, Clone)]
pub struct BezierBuilder<I, E, S, W> {
    input: I,
    elements: E,
    space: S,
    _phantom: PhantomData<*const W>,
}

impl Default for BezierBuilder<Unknown, Unknown, Unknown, Unknown> {
    fn default() -> Self {
        BezierBuilder::new()
    }
}

impl BezierBuilder<Unknown, Unknown, Unknown, Unknown> {
    /// Create a new bezier curve builder.
    pub const fn new() -> Self {
        BezierBuilder {
            input: Unknown,
            elements: Unknown,
            space: Unknown,
            _phantom: PhantomData,
        }
    }
}

impl BezierBuilder<Unknown, Unknown, Unknown, Unknown> {
    /// Create a new bezier curve builder.
    pub fn elements<E>(self, elements: E) -> BezierBuilder<Unknown, E, Unknown, WithoutWeight>
    where
        E: DiscreteGenerator,
    {
        BezierBuilder {
            input: self.input,
            space: self.space,
            elements,
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
    /// ```
    /// # use enterpolation::{bezier::{Bezier, BezierError}, Generator, Curve};
    /// # fn main() -> Result<(), BezierError> {
    /// let bez = Bezier::builder()
    ///                 .elements_with_weights([(1.0,1.0),(2.0,4.0),(3.0,0.0)])
    ///                 .normalized::<f64>()
    ///                 .constant()
    ///                 .build()?;
    /// let results = [1.0,15.0/8.25,10.0/4.5,19.0/6.25,f64::INFINITY];
    /// for (value,result) in bez.take(5).zip(results.iter().copied()){
    ///     assert_eq!(value, result);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn elements_with_weights<G>(
        self,
        gen: G,
    ) -> BezierBuilder<Unknown, Weights<G>, Unknown, WithWeight>
    where
        G: DiscreteGenerator,
        G::Output: IntoWeight,
        <G::Output as IntoWeight>::Element:
            Mul<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
        <G::Output as IntoWeight>::Weight: Zero + Copy,
    {
        BezierBuilder {
            input: self.input,
            space: self.space,
            elements: Weights::new(gen),
            _phantom: PhantomData,
        }
    }
}

impl<E, W> BezierBuilder<Unknown, E, Unknown, W> {
    /// Set the input type used for this interpolation and its domain to [0.0,1.0].
    pub fn normalized<R>(self) -> BezierBuilder<NormalizedInput<R>, E, Unknown, W> {
        BezierBuilder {
            input: NormalizedInput::new(),
            space: self.space,
            elements: self.elements,
            _phantom: self._phantom,
        }
    }
    /// Set the domain of the interpolation.
    pub fn domain<R>(self, start: R, end: R) -> BezierBuilder<InputDomain<R>, E, Unknown, W> {
        BezierBuilder {
            input: InputDomain::new(start, end),
            space: self.space,
            elements: self.elements,
            _phantom: self._phantom,
        }
    }
}

impl<I, E, W> BezierBuilder<I, E, Unknown, W>
where
    E: DiscreteGenerator,
{
    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder to use a vector as workspace,
    /// such you don't need to know the degree of the bezier curve at compile-time,
    /// but every generation of a value an allocation of memory will be necessary.
    #[cfg(feature = "std")]
    pub fn dynamic(self) -> BezierBuilder<I, E, DynSpace<E::Output>, W> {
        BezierBuilder {
            input: self.input,
            space: DynSpace::new(self.elements.len()),
            elements: self.elements,
            _phantom: self._phantom,
        }
    }

    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder the size of the workspace needed such that no memory allocations are needed
    /// when interpolating.
    pub fn constant<const N: usize>(self) -> BezierBuilder<I, E, ConstSpace<E::Output, N>, W>
    where
        E: ConstDiscreteGenerator<N>,
    {
        BezierBuilder {
            input: self.input,
            space: ConstSpace::new(),
            elements: self.elements,
            _phantom: self._phantom,
        }
    }

    /// Set the workspace whcih the interpolation uses.
    ///
    /// The workspace has to have a size of the number of elements in the bezier curve.
    ///
    /// If the degree of the bezier curve is known at compile-time, consider using `constant` instead.
    /// Otherwise without std support, one has to set a specific object implementing the `Space` trait.
    pub fn workspace<S>(self, space: S) -> BezierBuilder<I, E, S, W>
    where
        S: Space<E::Output>,
    {
        BezierBuilder {
            input: self.input,
            space,
            elements: self.elements,
            _phantom: self._phantom,
        }
    }
}

impl<R, E, S> BezierBuilder<NormalizedInput<R>, E, S, WithoutWeight>
where
    E: DiscreteGenerator,
    E::Output: Merge<R>,
    S: Space<E::Output>,
    R: Real,
{
    /// Build a bezier interpolation.
    pub fn build(self) -> Result<Bezier<R, E, S>, BezierError> {
        Bezier::new(self.elements, self.space)
    }
}

impl<R, E, S> BezierBuilder<InputDomain<R>, E, S, WithoutWeight>
where
    E: DiscreteGenerator,
    E::Output: Merge<R> + Copy,
    S: Space<E::Output>,
    R: Real,
{
    /// Build a bezier interpolation with the given domain
    #[allow(clippy::type_complexity)]
    pub fn build(self) -> Result<TransformInput<Bezier<R, E, S>, R, R>, BezierError> {
        Ok(TransformInput::normalized_to_domain(
            Bezier::new(self.elements, self.space)?,
            self.input.start,
            self.input.end,
        ))
    }
}

impl<R, G, S> BezierBuilder<NormalizedInput<R>, Weights<G>, S, WithWeight>
where
    R: Real + Copy,
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    S: Space<Homogeneous<<G::Output as IntoWeight>::Element, <G::Output as IntoWeight>::Weight>>,
    <Weights<G> as Generator<usize>>::Output: Merge<R>,
    <G::Output as IntoWeight>::Element:
        Div<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
{
    /// Build a weighted bezier interpolation.
    pub fn build(self) -> Result<WeightedBezier<R, G, S>, BezierError> {
        Ok(Weighted::new(Bezier::new(self.elements, self.space)?))
    }
}

impl<R, G, S> BezierBuilder<InputDomain<R>, Weights<G>, S, WithWeight>
where
    R: Real + Copy,
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    S: Space<Homogeneous<<G::Output as IntoWeight>::Element, <G::Output as IntoWeight>::Weight>>,
    <Weights<G> as Generator<usize>>::Output: Merge<R> + Copy,
    <G::Output as IntoWeight>::Element:
        Div<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
{
    /// Build a weighted bezier interpolation with given domain.
    #[allow(clippy::type_complexity)]
    pub fn build(self) -> Result<TransformInput<WeightedBezier<R, G, S>, R, R>, BezierError> {
        Ok(TransformInput::normalized_to_domain(
            Weighted::new(Bezier::new(self.elements, self.space)?),
            self.input.start,
            self.input.end,
        ))
    }
}

/// Type alias for weighted beziers.
type WeightedBezier<R, G, S> = Weighted<Bezier<R, Weights<G>, S>>;

#[cfg(test)]
mod test {
    use super::BezierBuilder;
    // Homogeneous for creating Homogeneous, Generator for using .stack()
    use crate::{weights::Homogeneous, Generator};
    #[test]
    fn elements_with_weights() {
        BezierBuilder::new()
            .elements_with_weights([(1.0, 1.0), (2.0, 2.0), (3.0, 0.0)])
            .normalized::<f64>()
            .constant()
            .build()
            .unwrap();
        BezierBuilder::new()
            .elements_with_weights([1.0, 2.0, 3.0].stack([1.0, 2.0, 0.0]))
            .normalized::<f64>()
            .constant()
            .build()
            .unwrap();
        BezierBuilder::new()
            .elements_with_weights([
                Homogeneous::new(1.0),
                Homogeneous::weighted_unchecked(2.0, 2.0),
                Homogeneous::infinity(3.0),
            ])
            .normalized::<f64>()
            .constant()
            .build()
            .unwrap();
        BezierBuilder::new()
            .elements([0.1, 0.2, 0.3])
            .normalized::<f64>()
            .constant()
            .build()
            .unwrap();
        assert!(BezierBuilder::new()
            .elements::<[f64; 0]>([])
            .normalized::<f64>()
            .constant()
            .build()
            .is_err());
    }
}
