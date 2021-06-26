//! Builder module for bspline interpolations.
//!
//! Each interpolation has it's own builder module, which accumalates all methods to create their interpolation.

//TODO: EXAMPLE

use core::ops::{Add, Mul, Div};
use core::marker::PhantomData;
use num_traits::real::Real;
use num_traits::FromPrimitive;
use num_traits::identities::Zero;
use crate::{Generator, DiscreteGenerator, ConstDiscreteGenerator, Weighted, Weights,
    IntoWeight, Homogeneous, Space, DynSpace, ConstSpace, Sorted, SortedGenerator, Equidistant};
use crate::builder::{WithWeight,WithoutWeight,Unknown, Output};
use super::BSpline;
use super::error::{Empty, BSplineError, NonStrictPositiveDegree};
// use super::error::{LinearError, ToFewElements, KnotElementInequality};

/// Marker Struct which saves a type and an usize.
///
/// Used to save the which of equidistant with a specific length.
#[derive(Debug, Clone)]
pub struct UnknownDomain<R> {
    _phantom: PhantomData<*const R>,
    len: usize,
}

impl<R> UnknownDomain<R>{
    pub fn new(len: usize) -> Self {
        UnknownDomain {
            _phantom: PhantomData,
            len,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
}

/// Builder for bspline interpolation.
///
/// This struct helps create bspline interpolations.
/// Usually one creates an instance by using the `builder()` method on the interpolation itself.
///
/// Before building, one has to give information for:
/// - The elements the interpolation should use. Methods like `elements` and `elements_with_weights`
/// exist for that cause.
/// - The knots the interpolation uses. Either by giving them directly with `knots` or by using
/// equidistant knots with `equidistant`.
/// - A workspace to use, that is, a mutable slice-like object to do operations on.
/// Usually this is done by calling `constant` or `dynamic`.
#[derive(Debug, Clone)]
pub struct BSplineBuilder<K,E,S,W> {
    elements: E,
    knots: K,
    space: S,
    _phantom: PhantomData<*const W>,
}

impl Default for BSplineBuilder<Unknown, Unknown, Unknown, Unknown> {
    fn default() -> Self {
        BSplineBuilder::new()
    }
}

impl BSplineBuilder<Unknown, Unknown, Unknown, Unknown> {
    /// Create a new linear interpolation builder.
    pub const fn new() -> Self {
        BSplineBuilder {
            elements: Unknown,
            knots: Unknown,
            space: Unknown,
            _phantom: PhantomData,
        }
    }
}

impl BSplineBuilder<Unknown, Unknown, Unknown, Unknown> {
    /// Set the elements of the bspline interpolation.
    pub fn elements<E>(self, elements: E) -> Result<BSplineBuilder<Unknown, E, Unknown, WithoutWeight>, Empty>
    where E: DiscreteGenerator,
    {
        if elements.is_empty() {
            return Err(Empty::new());
        }
        Ok(BSplineBuilder {
            knots: self.knots,
            space: self.space,
            elements,
            _phantom: PhantomData,
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
    pub fn elements_with_weights<G>(self, gen: G)
        -> Result<BSplineBuilder<Unknown, Weights<G>,Unknown, WithWeight>,Empty>
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
        Ok(BSplineBuilder {
            space: self.space,
            knots: self.knots,
            elements: Weights::new(gen),
            _phantom: PhantomData,
        })
    }
}

impl<E,W> BSplineBuilder<Unknown, E, Unknown, W>
{
    /// Set the knots of the interpolation.
    ///
    /// The degree of this bspline interplation is given by knots.len() - elements.len() - 1.
    /// Only degress of >= 1 are allowed.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using `equidistant()` instead.
    pub fn knots<K>(self, knots: K) -> Result<BSplineBuilder<Sorted<K>,E, Unknown, W>, BSplineError>
    where
        E: DiscreteGenerator,
        K: DiscreteGenerator,
        K::Output: PartialOrd
    {
        if knots.is_empty() || knots.len() - 1 <= self.elements.len()
        {
            return Err(NonStrictPositiveDegree::new(self.elements.len(), knots.len()).into());
        }
        Ok(BSplineBuilder {
            knots: Sorted::new(knots)?,
            elements: self.elements,
            space: self.space,
            _phantom: self._phantom,
        })
    }

    /// Build an interpolation with equidistant knots.
    pub fn equidistant<R>(self) -> BSplineBuilder<Output<R>,E, Unknown, W>{
        BSplineBuilder {
            knots: Output::new(),
            elements: self.elements,
            space: self.space,
            _phantom: self._phantom,
        }
    }
}

impl<R,E,W> BSplineBuilder<Output<R>, E, Unknown, W>
where
    E: DiscreteGenerator,
{
    /// Set the degree of the curve. The degree has to be bigger then 0, otherwise it will return an error.
    pub fn degree(self, degree: usize) -> Result<BSplineBuilder<UnknownDomain<R>,E,Unknown,W>,NonStrictPositiveDegree>{

        if degree == 0 {
            return Err(NonStrictPositiveDegree::new(self.elements.len() + degree + 1, self.elements.len()));
        }

        Ok(BSplineBuilder{
            knots: UnknownDomain::new(self.elements.len() + degree + 1),
            elements: self.elements,
            space: self.space,
            _phantom: self._phantom,
        })
    }
}

impl<R,E,W> BSplineBuilder<UnknownDomain<R>, E, Unknown, W>
where
    E: DiscreteGenerator,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(self, start: R, end: R) -> BSplineBuilder<Equidistant<R>,E,Unknown,W>{
        BSplineBuilder {
            knots: Equidistant::new(self.knots.len(), start, end),
            elements: self.elements,
            space: self.space,
            _phantom: self._phantom,
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(self) -> BSplineBuilder<Equidistant<R>,E,Unknown,W>{
        BSplineBuilder {
            knots: Equidistant::normalized(self.knots.len()),
            elements: self.elements,
            space: self.space,
            _phantom: self._phantom,
        }
    }
}

impl<K,E,W> BSplineBuilder<K,E, Unknown, W>
where E: DiscreteGenerator
{
    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder to use a vector as workspace,
    /// such you don't need to know the degree of the bezier curve at compile-time,
    /// but every generation of a value an allocation of memory will be necessary.
    pub fn dynamic(self) -> BSplineBuilder<K,E,DynSpace<E::Output>,W>{
        BSplineBuilder{
            knots: self.knots,
            space: DynSpace::new(self.elements.len()),
            elements: self.elements,
            _phantom: self._phantom,
        }
    }

    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder the size of the workspace needed such that no memory allocations are needed
    /// when interpolating.
    pub fn constant<const N: usize>(self) -> BSplineBuilder<K,E,ConstSpace<E::Output,N>,W>
    where E: ConstDiscreteGenerator<N>
    {
        BSplineBuilder{
            knots: self.knots,
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
    pub fn workspace<S>(self, space: S) -> BSplineBuilder<K,E,S,W>
    where S: Space<E::Output>
    {
        //TODO: return error instead of panic
        assert!(space.len() >= self.elements.len());

        BSplineBuilder{
            knots: self.knots,
            space,
            elements: self.elements,
            _phantom: self._phantom,
        }
    }
}

impl<K,E,S> BSplineBuilder<K,E,S, WithoutWeight>
where
    K: SortedGenerator,
    E: DiscreteGenerator,
    E::Output: Add<Output = E::Output> + Mul<K::Output, Output = E::Output> + Copy,
    S: Space<E::Output>,
{
    /// Build a bezier interpolation.
    pub fn build(self) -> BSpline<K,E,S>{
        BSpline::new_unchecked(self.elements, self.knots, self.space)
    }
}

impl<K,G,S> BSplineBuilder<K,Weights<G>,S,WithWeight>
where
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    K: SortedGenerator,
    S: Space<Homogeneous<<G::Output as IntoWeight>::Element, <G::Output as IntoWeight>::Weight>>,
    <Weights<G> as Generator<usize>>::Output:
        Add<Output = <Weights<G> as Generator<usize>>::Output> +
        Mul<K::Output, Output = <Weights<G> as Generator<usize>>::Output> +
        Copy,
    <G::Output as IntoWeight>::Element: Div<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
{
    /// Build a weighted bezier interpolation.
    pub fn build(self) -> Weighted<BSpline<K,Weights<G>,S>>
    {
        Weighted::new(BSpline::new_unchecked(self.elements, self.knots, self.space))
    }
}

// possible variations:
// elements (1) or elements_with_weights (3)

#[cfg(test)]
mod test {
    use super::BSplineBuilder;
    // Homogeneous for creating Homogeneous, Generator for using .stack()
    use crate::{Homogeneous, Generator};
    #[test]
    fn elements_with_weights() {
        BSplineBuilder::new()
            .elements_with_weights([(1.0,1.0),(2.0,2.0),(3.0,0.0)]).unwrap()
            .equidistant::<f64>()
            .degree(2).unwrap()
            .domain(0.0,5.0)
            .constant()
            .build();
        BSplineBuilder::new()
            .elements_with_weights([1.0,2.0,3.0].stack([1.0,2.0,0.0])).unwrap()
            .equidistant::<f64>()
            .degree(1).unwrap()
            .normalized()
            .constant()
            .build();
        BSplineBuilder::new()
            .elements_with_weights([
                Homogeneous::new(1.0),
                Homogeneous::weighted_unchecked(2.0, 2.0),
                Homogeneous::infinity(3.0)]).unwrap()
            .knots([0.0,1.0,2.0,3.0,4.0]).unwrap()
            .constant()
            .build();
        BSplineBuilder::new()
            .elements(vec![0.1,0.2,0.3]).unwrap()
            .equidistant::<f64>()
            .degree(1).unwrap()
            .normalized()
            .dynamic()
            .build();
        assert!(BSplineBuilder::new().elements::<[f64;0]>([]).is_err());
    }
}
