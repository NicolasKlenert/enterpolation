//! Builder module for linear interpolations.

use super::error::LinearError;
use super::{KnotElementInequality, Linear, TooFewElements};
use crate::builder::{Type, Unknown, WithWeight, WithoutWeight};
use crate::weights::{IntoWeight, Weighted, Weights};
use crate::{Chain, Equidistant, Identity, Signal, Sorted, SortedChain};
use core::marker::PhantomData;
use core::ops::Mul;
use num_traits::FromPrimitive;
use num_traits::identities::Zero;
use num_traits::real::Real;
use topology_traits::Merge;

/// Builder for linear interpolation.
///
/// This struct helps create linear interpolations. The difference between this struct and [`LinearBuilder`]
/// is that this struct may have other fallible methods and not only the [`build()`] method.
///
/// Before building, one has to give information for:
/// - The elements the interpolation should use. Methods like [`elements()`] and [`elements_with_weights()`]
///   exist for that cause.
/// - The knots the interpolation uses. This can be seen as the spacing between those elements.
///   Either by giving them directly with [`knots()`] or by using equidistant knots with [`equidistant()`].
///
/// ```rust
/// # use enterpolation::{linear::{LinearDirector, LinearError}, Signal, Curve};
/// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
/// #
/// # fn main() -> Result<(), LinearError> {
/// let linear = LinearDirector::new()
///                 .elements([1.0,5.0,100.0])?
///                 .equidistant::<f64>()
///                 .normalized()
///                 .build();
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LinearDirector<K, E, F, W> {
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
/// # use enterpolation::{linear::{Linear, LinearError}, Signal, Curve};
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct LinearBuilder<K, E, F, W> {
    inner: Result<LinearDirector<K, E, F, W>, LinearError>,
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
            inner: Ok(LinearDirector::new()),
        }
    }
}

impl<F> LinearDirector<Unknown, Unknown, F, Unknown> {
    /// Set the elements of the linear interpolation.
    ///
    /// # Errors
    ///
    /// Returns [`TooFewElements`] if not at least 2 elements are given.
    ///
    /// [`TooFewElements`]: super::error::LinearError
    pub fn elements<E>(
        self,
        elements: E,
    ) -> Result<LinearDirector<Unknown, E, F, WithoutWeight>, TooFewElements>
    where
        E: Chain,
    {
        if elements.len() < 2 {
            return Err(TooFewElements::new(elements.len()));
        }
        Ok(LinearDirector {
            knots: self.knots,
            elements,
            easing: self.easing,
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
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use enterpolation::{linear::{Linear, LinearError}, Signal, Curve};
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
    ///
    /// # Errors
    ///
    /// Returns [`TooFewElements`] if not at least 2 elements are given.
    ///
    /// [`TooFewElements`]: super::error::LinearError
    pub fn elements_with_weights<G>(
        self,
        signal: G,
    ) -> Result<LinearDirector<Unknown, Weights<G>, F, WithWeight>, TooFewElements>
    where
        G: Chain,
        G::Output: IntoWeight,
        <G::Output as IntoWeight>::Element:
            Mul<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
        <G::Output as IntoWeight>::Weight: Zero + Copy,
    {
        if signal.len() < 2 {
            return Err(TooFewElements::new(signal.len()));
        }
        Ok(LinearDirector {
            knots: self.knots,
            elements: Weights::new(signal),
            easing: self.easing,
            _phantom: PhantomData,
        })
    }
}

impl<F> LinearBuilder<Unknown, Unknown, F, Unknown> {
    /// Set the elements of the linear interpolation.
    pub fn elements<E>(self, elements: E) -> LinearBuilder<Unknown, E, F, WithoutWeight>
    where
        E: Chain,
    {
        LinearBuilder {
            inner: self
                .inner
                .and_then(|director| director.elements(elements).map_err(|err| err.into())),
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
    /// # use enterpolation::{linear::{Linear, LinearError}, Signal, Curve};
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
    pub fn elements_with_weights<G>(
        self,
        signal: G,
    ) -> LinearBuilder<Unknown, Weights<G>, F, WithWeight>
    where
        G: Chain,
        G::Output: IntoWeight,
        <G::Output as IntoWeight>::Element:
            Mul<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
        <G::Output as IntoWeight>::Weight: Zero + Copy,
    {
        LinearBuilder {
            inner: self.inner.and_then(|director| {
                director
                    .elements_with_weights(signal)
                    .map_err(|err| err.into())
            }),
        }
    }
}

impl<E, F, W> LinearDirector<Unknown, E, F, W> {
    /// Set the knots of the interpolation.
    ///
    /// The amount of knots must be equal to the amount of elements.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using [`equidistant()`] instead.
    ///
    /// [`equidistant()`]: LinearDirector::equidistant()
    ///
    /// # Errors
    ///
    /// Returns [`KnotElementInequality`] if the number of knots is not equal to the number of elements.
    /// Returns [`NotSorted`] if the knots are not sorted such that they are increasing.
    ///
    /// [`KnotElementInequality`]: super::error::LinearError
    /// [`NotSorted`]:  super::error::LinearError
    pub fn knots<K>(self, knots: K) -> Result<LinearDirector<Sorted<K>, E, F, W>, LinearError>
    where
        E: Chain,
        K: Chain,
        K::Output: PartialOrd,
    {
        if self.elements.len() != knots.len() {
            return Err(KnotElementInequality::new(self.elements.len(), knots.len()).into());
        }
        Ok(LinearDirector {
            knots: Sorted::new(knots)?,
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        })
    }

    /// Build an interpolation with equidistant knots.
    ///
    /// This method takes `R` as a generic parameter. `R` has to be the type you want the knots to be.
    /// Often this is just `f32` or `f64`.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Performance
    ///
    /// This may drastically increase performance, as one does not have to use binary search to find
    /// the relevant knots in an interpolation.
    ///
    /// [`domain()`]: LinearDirector::domain()
    /// [`normalized()`]: LinearDirector::normalized()
    /// [`distance()`]: LinearDirector::distance()
    pub fn equidistant<R>(self) -> LinearDirector<Type<R>, E, F, W> {
        LinearDirector {
            knots: Type::new(),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }
}

impl<E, F, W> LinearBuilder<Unknown, E, F, W> {
    /// Set the knots of the interpolation.
    ///
    /// The amount of knots must be equal to the amount of elements.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using [`equidistant()`] instead.
    ///
    /// [`equidistant()`]: LinearBuilder::equidistant()
    pub fn knots<K>(self, knots: K) -> LinearBuilder<Sorted<K>, E, F, W>
    where
        E: Chain,
        K: Chain,
        K::Output: PartialOrd,
    {
        LinearBuilder {
            inner: self.inner.and_then(|director| director.knots(knots)),
        }
    }

    /// Build an interpolation with equidistant knots.
    ///
    /// This method takes `R` as a generic parameter. `R` has to be the type you want the knots to be.
    /// Often this is just `f32` or `f64`.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Performance
    ///
    /// This may drastically increase performance, as one does not have to use binary search to find
    /// the relevant knots in an interpolation.
    ///
    /// [`domain()`]: LinearBuilder::domain()
    /// [`normalized()`]: LinearBuilder::normalized()
    /// [`distance()`]: LinearBuilder::distance()
    pub fn equidistant<R>(self) -> LinearBuilder<Type<R>, E, F, W> {
        LinearBuilder {
            inner: self.inner.map(|director| director.equidistant()),
        }
    }
}

impl<R, E, F, W> LinearDirector<Type<R>, E, F, W>
where
    E: Chain,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(self, start: R, end: R) -> LinearDirector<Equidistant<R>, E, F, W> {
        LinearDirector {
            knots: Equidistant::new(self.elements.len(), start, end),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(self) -> LinearDirector<Equidistant<R>, E, F, W> {
        LinearDirector {
            knots: Equidistant::normalized(self.elements.len()),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots
    pub fn distance(self, start: R, step: R) -> LinearDirector<Equidistant<R>, E, F, W> {
        LinearDirector {
            knots: Equidistant::step(self.elements.len(), start, step),
            elements: self.elements,
            easing: self.easing,
            _phantom: self._phantom,
        }
    }
}

impl<R, E, F, W> LinearBuilder<Type<R>, E, F, W>
where
    E: Chain,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(self, start: R, end: R) -> LinearBuilder<Equidistant<R>, E, F, W> {
        LinearBuilder {
            inner: self.inner.map(|director| director.domain(start, end)),
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(self) -> LinearBuilder<Equidistant<R>, E, F, W> {
        LinearBuilder {
            inner: self.inner.map(|director| director.normalized()),
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots
    pub fn distance(self, start: R, step: R) -> LinearBuilder<Equidistant<R>, E, F, W> {
        LinearBuilder {
            inner: self.inner.map(|director| director.distance(start, step)),
        }
    }
}

impl<K, E, F, W> LinearDirector<K, E, F, W>
where
    K: SortedChain,
{
    /// Sets an easing function.
    ///
    /// This allows quasi-linear interpolations. Before merging two elements together with a factor,
    /// the factor is send to the given function before and the output is the new factor.
    ///
    /// # Examples
    ///
    /// See the [plateau example] for more information.
    ///
    /// [plateau example]: https://github.com/NicolasKlenert/enterpolation/blob/main/examples/plateaus.rs
    pub fn easing<FF>(self, easing: FF) -> LinearDirector<K, E, FF, W> {
        LinearDirector {
            knots: self.knots,
            elements: self.elements,
            easing,
            _phantom: self._phantom,
        }
    }
}

impl<K, E, F, W> LinearBuilder<K, E, F, W>
where
    K: SortedChain,
{
    /// Sets an easing function.
    ///
    /// This allows quasi-linear interpolations. Before merging two elements together with a factor,
    /// the factor is send to the given function before and the output is the new factor.
    ///
    /// # Examples
    ///
    /// See the [plateau example] for more information.
    ///
    /// [plateau example]: https://github.com/NicolasKlenert/enterpolation/blob/main/examples/plateaus.rs
    pub fn easing<FF>(self, easing: FF) -> LinearBuilder<K, E, FF, W> {
        LinearBuilder {
            inner: self.inner.map(|director| director.easing(easing)),
        }
    }
}

impl<K, E, F> LinearDirector<K, E, F, WithoutWeight>
where
    E: Chain,
    K: SortedChain,
    E::Output: Merge<K::Output>,
    K::Output: Real,
{
    /// Build a linear interpolation.
    pub fn build(self) -> Linear<K, E, F> {
        Linear::new_unchecked(self.elements, self.knots, self.easing)
    }
}

impl<K, E, F> LinearBuilder<K, E, F, WithoutWeight>
where
    E: Chain,
    K: SortedChain,
    E::Output: Merge<K::Output>,
    K::Output: Real,
{
    /// Build a linear interpolation.
    pub fn build(self) -> Result<Linear<K, E, F>, LinearError> {
        match self.inner {
            Err(err) => Err(err),
            Ok(director) => Ok(director.build()),
        }
    }
}

impl<K, G, F> LinearDirector<K, Weights<G>, F, WithWeight>
where
    K: SortedChain,
    K::Output: Real + Copy,
    G: Chain,
    G::Output: IntoWeight,
    <Weights<G> as Signal<usize>>::Output: Merge<K::Output>,
{
    /// Build a weighted linear interpolation.
    pub fn build(self) -> WeightedLinear<K, G, F> {
        Weighted::new(Linear::new_unchecked(
            self.elements,
            self.knots,
            self.easing,
        ))
    }
}

impl<K, G, F> LinearBuilder<K, Weights<G>, F, WithWeight>
where
    K: SortedChain,
    K::Output: Real + Copy,
    G: Chain,
    G::Output: IntoWeight,
    <Weights<G> as Signal<usize>>::Output: Merge<K::Output>,
{
    /// Build a weighted linear interpolation.
    pub fn build(self) -> Result<WeightedLinear<K, G, F>, LinearError> {
        match self.inner {
            Err(err) => Err(err),
            Ok(director) => Ok(director.build()),
        }
    }
}

/// Type alias for weighted linear interpolations
type WeightedLinear<K, G, F> = Weighted<Linear<K, Weights<G>, F>>;

#[cfg(test)]
mod test {
    use super::LinearBuilder;
    // Homogeneous for creating Homogeneous, Signal for using .stack()
    use crate::{Signal, linear::LinearDirector, weights::Homogeneous};
    #[test]
    fn building_weights() {
        LinearBuilder::new()
            .elements_with_weights([(1.0, 1.0), (2.0, 2.0), (3.0, 0.0)])
            .equidistant::<f64>()
            .normalized()
            .build()
            .unwrap();
        LinearBuilder::new()
            .elements_with_weights([1.0, 2.0, 3.0].stack([1.0, 2.0, 0.0]))
            .equidistant::<f64>()
            .normalized()
            .build()
            .unwrap();
        LinearBuilder::new()
            .elements_with_weights([
                Homogeneous::new(1.0),
                Homogeneous::weighted_unchecked(2.0, 2.0),
                Homogeneous::infinity(3.0),
            ])
            .knots([1.0, 2.0, 3.0])
            .build()
            .unwrap();
        LinearBuilder::new()
            .elements([0.1, 0.2, 0.3])
            .equidistant::<f64>()
            .normalized()
            .build()
            .unwrap();
    }

    #[test]
    fn builder_errors() {
        assert!(
            LinearBuilder::new()
                .elements::<[f64; 0]>([])
                .knots::<[f64; 0]>([])
                .build()
                .is_err()
        );
        assert!(
            LinearBuilder::new()
                .elements([1.0])
                .knots([1.0])
                .build()
                .is_err()
        );
        assert!(
            LinearBuilder::new()
                .elements([1.0, 2.0])
                .knots([1.0, 2.0, 3.0])
                .build()
                .is_err()
        );
    }

    #[test]
    fn director_errors() {
        assert!(LinearDirector::new().elements([0.0]).is_err());
        assert!(
            LinearDirector::new()
                .elements([0.0, 1.0])
                .unwrap()
                .knots([1.0])
                .is_err()
        );
        assert!(
            LinearDirector::new()
                .elements([1.0, 2.0])
                .unwrap()
                .knots([1.0, 2.0, 3.0])
                .is_err()
        );
        assert!(
            LinearDirector::new()
                .elements([1.0, 2.0])
                .unwrap()
                .knots([1.0, 2.0])
                .is_ok()
        );
    }
}
