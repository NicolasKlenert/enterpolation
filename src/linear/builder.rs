//! Builder module for linear interpolations.

use core::ops::Mul;
use core::marker::PhantomData;
use num_traits::real::Real;
use num_traits::FromPrimitive;
use num_traits::identities::Zero;
use topology_traits::Merge;
use crate::{Generator, DiscreteGenerator, SortedGenerator, Sorted, NotSorted, Equidistant, Identity};
use crate::weights::{Weighted, Weights, IntoWeight};
use crate::builder::{WithWeight,WithoutWeight, Type, Unknown};
use super::Linear;
use super::error::LinearError;

/// Builder for linear interpolation.
///
/// This struct helps create linear interpolations. The differene between this struct and [`LinearBuilder`]
/// is that this struct may have other fallible methods and not only the [`build()`] method.
///
/// Before building, one has to give information for:
/// - The elements the interpolation should use. Methods like [`elements()`] and [`elements_with_weights()`]
///   exist for that cause.
/// - The knots the interpolation uses. This can be seen as the spacing between those elements.
///   Either by giving them directly with [`knots()`] or by using equidistant knots with [`equidistant()`].
///
/// ```rust
/// # use enterpolation::{linear::{LinearDirector, LinearError}, Generator, Curve};
/// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
/// #
/// # fn main() -> Result<(), LinearError> {
/// let linear = LinearDirector::new()
///                 .elements([1.0,5.0,100.0])
///                 .equidistant::<f64>()
///                 .normalized()
///                 .build()?;
/// let results = [1.0,3.0,5.0,52.5,100.0];
/// for (value,result) in linear.take(5).zip(results.iter().copied()){
///     assert_f64_near!(value, result);
/// }
/// #
/// #     Ok(())
/// # }
/// ```
///
/// Sometimes the spacing between elements should not also be linear, such creating a quasi-linear interpolation.
/// To achieve this, one may use the [`easing()`] function.
/// A working example of this can be seen in [plateaus.rs].
///
/// Linear equidistant constant interpolations are often wanted to define some specific curve
/// (like a specific gradient). To create such interpolation, the builder pattern can not be used yet.
/// Instead one should create a linear interpolation directly with the [`equidistant_unchecked()`] constructor.
///
/// [`LinearBuilder`]: LinearBuilder
/// [`easing()`]: LinearDirector::easing()
/// [plateaus.rs]: https://github.com/NicolasKlenert/enterpolation/blob/main/examples/plateaus.rs
/// [`build()`]: LinearDirector::build()
/// [`elements()`]: LinearDirector::elements()
/// [`elements_with_weights()`]: LinearDirector::elements_with_weights()
/// [`knots()`]: LinearDirector::knots()
/// [`equidistant()`]: LinearDirector::equidistant()
/// [`equidistant_unchecked()`]: super::Linear::equidistant_unchecked()
#[derive(Debug, Clone)]
pub struct LinearDirector<K,E,F,W> {
    knots: K,
    elements: E,
    easing: F,
    _phantom: PhantomData<*const W>,
}

/// Builder for linear interpolation.
///
/// This struct helps create linear interpolations. Its only fallible method is [`build()`].
/// Usually one creates an instance by using the [`builder()`] method on the interpolation itself.
///
/// Before building, one has to give information for:
/// - The elements the interpolation should use. Methods like [`elements()`] and [`elements_with_weights()`]
///   exist for that cause.
/// - The knots the interpolation uses. This can be seen as the spacing between those elements.
///   Either by giving them directly with [`knots()`] or by using equidistant knots with [`equidistant()`].
///
/// ```rust
/// # use enterpolation::{linear::{Linear, LinearError}, Generator, Curve};
/// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
/// #
/// # fn main() -> Result<(), LinearError> {
/// let linear = Linear::builder()
///                 .elements([1.0,5.0,100.0])
///                 .equidistant::<f64>()
///                 .normalized()
///                 .build()?;
/// let results = [1.0,3.0,5.0,52.5,100.0];
/// for (value,result) in linear.take(5).zip(results.iter().copied()){
///     assert_f64_near!(value, result);
/// }
/// #
/// #     Ok(())
/// # }
/// ```
///
/// Sometimes the spacing between elements should not also be linear, such creating a quasi-linear interpolation.
/// To achieve this, one may use the [`easing()`] function.
/// A working example of this can be seen in [plateaus.rs].
///
/// Linear equidistant constant interpolations are often wanted to define some specific curve
/// (like a specific gradient). To create such interpolation, the builder pattern can not be used yet.
/// Instead one should create a linear interpolation directly with the [`equidistant_unchecked()`] constructor.
///
/// [`easing()`]: LinearBuilder::easing()
/// [plateaus.rs]: https://github.com/NicolasKlenert/enterpolation/blob/main/examples/plateaus.rs
/// [`build()`]: LinearBuilder::build()
/// [`builder()`]: super::Linear::builder()
/// [`elements()`]: LinearBuilder::elements()
/// [`elements_with_weights()`]: LinearBuilder::elements_with_weights()
/// [`knots()`]: LinearBuilder::knots()
/// [`equidistant()`]: LinearBuilder::equidistant()
/// [`equidistant_unchecked()`]: super::Linear::equidistant_unchecked()
#[derive(Debug, Clone)]
pub struct LinearBuilder<K,E,F,W> {
    inner: Result<LinearDirector<K,E,F,W>,LinearError>,
}

impl Default for LinearDirector<Unknown, Unknown, Identity, Unknown> {
    fn default() -> Self {
        LinearDirector::new()
    }
}

impl Default for LinearBuilder<Unknown, Unknown, Identity, Unknown> {
    fn default() -> Self {
        LinearBuilder::new()
    }
}

impl LinearDirector<Unknown, Unknown, Identity, Unknown> {
    /// Create a new linear interpolation builder.
    pub const fn new() -> Self {
        LinearDirector {
            knots: Unknown,
            elements: Unknown,
            easing: Identity::new(),
            _phantom: PhantomData,
        }
    }
}

impl LinearBuilder<Unknown, Unknown, Identity, Unknown> {
    /// Create a new linear interpolation builder.
    pub const fn new() -> Self {
        LinearBuilder {
            inner: Ok(LinearDirector::new())
        }
    }
}

impl<F> LinearDirector<Unknown, Unknown, F, Unknown> {
    /// Set the elements of the linear interpolation.
    pub fn elements<E>(self, elements: E) -> LinearDirector<Unknown, E, F, WithoutWeight>
    where E: DiscreteGenerator,
    {
        LinearDirector {
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
    pub fn elements_with_weights<G>(self, gen: G) -> LinearDirector<Unknown, Weights<G>, F, WithWeight>
    where
        G: DiscreteGenerator,
        G::Output: IntoWeight,
        <G::Output as IntoWeight>::Element:
            Mul<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
        <G::Output as IntoWeight>::Weight: Zero + Copy,
    {
        LinearDirector {
            knots: self.knots,
            elements: Weights::new(gen),
            easing: self.easing,
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
            inner: self.inner.and_then(|director| Ok(director.elements(elements)))
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
            inner: self.inner.and_then(|director| Ok(director.elements_with_weights(gen)))
        }
    }
}

impl<E,F,W> LinearDirector<Unknown, E, F, W>
{
    /// Set the knots of the interpolation.
    ///
    /// The amount of knots must be equal to the amount of elements.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using `equidistant()` instead.
    pub fn knots<K>(self, knots: K) -> Result<LinearDirector<Sorted<K>,E, F, W>, NotSorted>
    where
        E: DiscreteGenerator,
        K: DiscreteGenerator,
        K::Output: PartialOrd
    {
        Ok(LinearDirector {
            knots: Sorted::new(knots)?,
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        })
    }

    /// Build an interpolation with equidistant knots.
    ///
    /// This may drastically increase performance, as one does not have to use binary search to find
    /// the relevant knots in an interpolation.
    pub fn equidistant<R>(self) -> LinearDirector<Type<R>,E,F,W>{
        LinearDirector {
            knots: Type::new(),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
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
    pub fn knots<K>(self, knots: K) -> LinearBuilder<Sorted<K>,E, F, W>
    where
        E: DiscreteGenerator,
        K: DiscreteGenerator,
        K::Output: PartialOrd
    {
        LinearBuilder {
            inner: self.inner.and_then(|director| director.knots(knots).map_err(|err| err.into()))
        }
    }

    /// Build an interpolation with equidistant knots.
    ///
    /// This may drastically increase performance, as one does not have to use binary search to find
    /// the relevant knots in an interpolation.
    pub fn equidistant<R>(self) -> LinearBuilder<Type<R>,E,F,W>{
        LinearBuilder {
            inner: self.inner.and_then(|director| Ok(director.equidistant()))
        }
    }
}

impl<R,E,F,W> LinearDirector<Type<R>,E,F,W>
where
    E: DiscreteGenerator,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(self, start: R, end: R) -> LinearDirector<Equidistant<R>,E,F,W>{
        LinearDirector {
            knots: Equidistant::new(self.elements.len(), start, end),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(self) -> LinearDirector<Equidistant<R>,E,F,W>{
        LinearDirector {
            knots: Equidistant::normalized(self.elements.len()),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots
    pub fn distance(self, start: R, step: R) -> LinearDirector<Equidistant<R>,E,F,W>{
        LinearDirector {
            knots: Equidistant::step(self.elements.len(), start, step),
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
            inner: self.inner.and_then(|director| Ok(director.domain(start,end)))
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(self) -> LinearBuilder<Equidistant<R>,E,F,W>{
        LinearBuilder {
            inner: self.inner.and_then(|director| Ok(director.normalized()))
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots
    pub fn distance(self, start: R, step: R) -> LinearBuilder<Equidistant<R>,E,F,W>{
        LinearBuilder {
            inner: self.inner.and_then(|director| Ok(director.distance(start,step)))
        }
    }
}

impl<K,E,F,W> LinearDirector<K,E,F,W>
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
    pub fn easing<FF>(self, easing: FF) -> LinearDirector<K,E,FF,W>{
        LinearDirector {
            knots: self.knots,
            elements: self.elements,
            easing,
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
            inner: self.inner.and_then(|director| Ok(director.easing(easing)))
        }
    }
}

impl<K,E,F> LinearDirector<K,E,F,WithoutWeight>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    E::Output: Merge<K::Output>,
    K::Output: Real
{
    /// Build a linear interpolation.
    pub fn build(self) -> Result<Linear<K,E,F>,LinearError>{
        Linear::new(self.elements, self.knots, self.easing)
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
        match self.inner {
            Err(err)=> return Err(err),
            Ok(director) => director.build()
        }
    }
}

impl<K,G,F> LinearDirector<K,Weights<G>,F,WithWeight>
where
    K: SortedGenerator,
    K::Output: Real + Copy,
    G: DiscreteGenerator,
    G::Output: IntoWeight,
    <Weights<G> as Generator<usize>>::Output: Merge<K::Output>,
{
    /// Build a weighted linear interpolation.
    pub fn build(self) -> Result<WeightedLinear<K,G,F>, LinearError>
    {
        Ok(Weighted::new(Linear::new(self.elements, self.knots, self.easing)?))
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
    pub fn build(self) -> Result<WeightedLinear<K,G,F>, LinearError>
    {
        match self.inner {
            Err(err)=> return Err(err),
            Ok(director) => director.build()
        }
    }
}

/// Type alias for weighted linear interpolations
type WeightedLinear<K,G,F> = Weighted<Linear<K,Weights<G>,F>>;

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
