//! Builder module for bspline interpolations.
//!
//! Each interpolation has it's own builder module, which accumalates all methods to create their interpolation.

use super::adaptors::{BorderBuffer, BorderDeletion};
use super::error::{
    BSplineError, IncongruousElementsDegree, IncongruousElementsKnots, InvalidDegree, TooFewKnots,
};
use super::{BSpline, TooFewElements, TooSmallWorkspace};
#[cfg(feature = "std")]
use crate::DynSpace;
use crate::builder::{Type, Unknown, WithWeight, WithoutWeight};
use crate::weights::{Homogeneous, IntoWeight, Weighted, Weights};
use crate::{Chain, ConstSpace, Equidistant, Signal, Sorted, SortedChain, Space};
use core::marker::PhantomData;
use core::ops::{Div, Mul};
use num_traits::FromPrimitive;
use num_traits::identities::Zero;
use num_traits::real::Real;
use topology_traits::Merge;
// use super::error::{LinearError, ToFewElements, KnotElementInequality};

/// Marker struct to signify the building of a closed curve.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Clamped;
/// Marker struct to signify the building of an open or generic curve.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Open;
/// Marker struct to signify the building of a curve with knots in the usual configuration.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Legacy;
// #[derive(Debug, Clone, Copy)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
// pub struct Closed;

/// Marker Struct which saves data for equidistant.
///
/// This struct has len and degree, which are dependent on each other.
/// However the equation between these two variables changes, depending on the specifics of the curve.
/// Such, both should be calculated.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct UnknownDomain<R> {
    _phantom: PhantomData<*const R>,
    len: usize,
    deg: usize,
}

impl<R> UnknownDomain<R> {
    pub fn new(len: usize, deg: usize) -> Self {
        UnknownDomain {
            _phantom: PhantomData,
            len,
            deg,
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn deg(&self) -> usize {
        self.deg
    }
}

/// Builder for bspline interpolation.
///
/// This struct helps create bspline interpolations. The difference between this struct and [`BSplineBuilder`]
/// is that this struct is allowed to have fallible methods which are not [`build()`].
///
/// Before building, one has to give information for:
/// - The elements the interpolation should use. Methods like [`elements()`] and [`elements_with_weights`()]
///   exist for that cause.
/// - The knots the interpolation uses. Either by giving them directly with [`knots()`] or by using
///   equidistant knots with [`equidistant()`].
/// - A workspace to use, that is, a mutable slice-like object to do operations on.
///   Usually this is done by calling [`constant()`] or [`dynamic()`].
///   [`workspace()`] is also posbbile for a custom workspace.
///
/// Furthermore one may want to use different modes, toggled by the methods [`open()`],[`clamped()`]
/// and [`legacy()`], where [`open()`] is the default one.
///
/// [`build()`]: BSplineDirector::build()
/// [`BSplineBuilder`]: BSplineBuilder
/// [`elements()`]: BSplineDirector::elements()
/// [`elements_with_weights`()]: BSplineDirector::elements_with_weights()
/// [`knots()`]: BSplineDirector::knots()
/// [`equidistant()`]: BSplineDirector::equidistant()
/// [`constant()`]: BSplineDirector::constant()
/// [`dynamic()`]: BSplineDirector::dynamic()
/// [`workspace()`]: BSplineDirector::workspace()
/// [`open()`]: BSplineDirector::open()
/// [`clamped()`]: BSplineDirector::clamped()
/// [`legacy()`]: BSplineDirector::legacy()
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BSplineDirector<K, E, S, W, M> {
    elements: E,
    knots: K,
    space: S,
    _phantoms: (PhantomData<*const W>, PhantomData<*const M>),
}

/// Builder for bspline interpolation.
///
/// This struct helps create bspline interpolations. This is the usual builder to use
/// as the only fallible method is the [`build()`] method.
/// Usually one creates an instance by using the [`builder()`] method on the interpolation itself.
///
/// Before building, one has to give information for:
/// - The elements the interpolation should use. Methods like [`elements()`] and [`elements_with_weights`()]
///   exist for that cause.
/// - The knots the interpolation uses. Either by giving them directly with [`knots()`] or by using
///   equidistant knots with [`equidistant()`].
/// - A workspace to use, that is, a mutable slice-like object to do operations on.
///   Usually this is done by calling [`constant()`] or [`dynamic()`].
///   [`workspace()`] is also posbbile for a custom workspace.
///
/// Furthermore one may want to use different modes, toggled by the methods [`open()`],[`clamped()`]
/// and [`legacy()`], where [`open()`] is the default one.
///
/// [`build()`]: BSplineBuilder::build()
/// [`builder()`]: super::BSpline::builder()
/// [`elements()`]: BSplineBuilder::elements()
/// [`elements_with_weights`()]: BSplineBuilder::elements_with_weights()
/// [`knots()`]: BSplineBuilder::knots()
/// [`equidistant()`]: BSplineBuilder::equidistant()
/// [`constant()`]: BSplineBuilder::constant()
/// [`dynamic()`]: BSplineBuilder::dynamic()
/// [`workspace()`]: BSplineBuilder::workspace()
/// [`open()`]: BSplineBuilder::open()
/// [`clamped()`]: BSplineBuilder::clamped()
/// [`legacy()`]: BSplineBuilder::legacy()
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct BSplineBuilder<K, E, S, W, M> {
    inner: Result<BSplineDirector<K, E, S, W, M>, BSplineError>,
}

impl Default for BSplineDirector<Unknown, Unknown, Unknown, Unknown, Open> {
    fn default() -> Self {
        BSplineDirector::new()
    }
}

impl Default for BSplineBuilder<Unknown, Unknown, Unknown, Unknown, Open> {
    fn default() -> Self {
        BSplineBuilder::new()
    }
}

impl BSplineDirector<Unknown, Unknown, Unknown, Unknown, Open> {
    /// Create a new linear interpolation builder.
    pub const fn new() -> Self {
        BSplineDirector {
            elements: Unknown,
            knots: Unknown,
            space: Unknown,
            _phantoms: (PhantomData, PhantomData),
        }
    }
}

impl BSplineBuilder<Unknown, Unknown, Unknown, Unknown, Open> {
    /// Create a new linear interpolation builder.
    pub const fn new() -> Self {
        BSplineBuilder {
            inner: Ok(BSplineDirector::new()),
        }
    }
}

impl<M> BSplineDirector<Unknown, Unknown, Unknown, Unknown, M> {
    /// Change the mode to an open curve.
    pub fn open(self) -> BSplineDirector<Unknown, Unknown, Unknown, Unknown, Open> {
        BSplineDirector {
            knots: self.knots,
            space: self.space,
            elements: self.elements,
            _phantoms: (self._phantoms.0, PhantomData),
        }
    }

    /// Change the mode to a clamped curve.
    pub fn clamped(self) -> BSplineDirector<Unknown, Unknown, Unknown, Unknown, Clamped> {
        BSplineDirector {
            knots: self.knots,
            space: self.space,
            elements: self.elements,
            _phantoms: (self._phantoms.0, PhantomData),
        }
    }

    /// Change the mode to legacy.
    ///
    /// This allows the builder to take in the "usual" definition of knots for B-splines.
    /// For more information, see the *Peculariaty of B-splines* section of the [main documentation].
    ///
    /// [main documentation]: crate#b-spline-peculiarity
    pub fn legacy(self) -> BSplineDirector<Unknown, Unknown, Unknown, Unknown, Legacy> {
        BSplineDirector {
            knots: self.knots,
            space: self.space,
            elements: self.elements,
            _phantoms: (self._phantoms.0, PhantomData),
        }
    }

    // /// Ensure the curve to be a loop, that is, its start and end point are equal and have a smooth transition.
    // ///
    // /// This method changes the underlying knot and element chain, by repeating some.
    // pub fn loop(self) -> BSplineDirector<K,E, Unknown, W>{
    //
    // }

    /// Set the elements of the bspline interpolation.
    ///
    /// # Errors
    ///
    /// Returns [`TooFewElements`] if not at least 2 elements are given.
    ///
    /// [`TooFewElements`]: super::error::BSplineError
    pub fn elements<E>(
        self,
        elements: E,
    ) -> Result<BSplineDirector<Unknown, E, Unknown, WithoutWeight, M>, TooFewElements>
    where
        E: Chain,
    {
        if elements.len() < 2 {
            return Err(TooFewElements::new(elements.len()));
        }
        Ok(BSplineDirector {
            knots: self.knots,
            space: self.space,
            elements,
            _phantoms: (PhantomData, self._phantoms.1),
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
    /// # Errors
    ///
    /// Returns [`TooFewElements`] if not at least 2 elements are given.
    ///
    /// [`TooFewElements`]: super::error::BSplineError
    pub fn elements_with_weights<G>(
        self,
        signal: G,
    ) -> Result<BSplineDirector<Unknown, Weights<G>, Unknown, WithWeight, M>, TooFewElements>
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
        Ok(BSplineDirector {
            space: self.space,
            knots: self.knots,
            elements: Weights::new(signal),
            _phantoms: (PhantomData, self._phantoms.1),
        })
    }
}

impl<M> BSplineBuilder<Unknown, Unknown, Unknown, Unknown, M> {
    /// Change the mode to an open curve.
    pub fn open(self) -> BSplineBuilder<Unknown, Unknown, Unknown, Unknown, Open> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.open()),
        }
    }

    /// Change the mode to a clamped curve.
    pub fn clamped(self) -> BSplineBuilder<Unknown, Unknown, Unknown, Unknown, Clamped> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.clamped()),
        }
    }

    /// Change the mode to legacy.
    ///
    /// This allows the builder to take in the "usual" definition of knots for B-splines.
    /// For more information, see the *Peculariaty of B-splines* section of the [main documentation].
    ///
    /// [main documentation]: crate
    pub fn legacy(self) -> BSplineBuilder<Unknown, Unknown, Unknown, Unknown, Legacy> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.legacy()),
        }
    }

    // /// Ensure the curve to be a loop, that is, its start and end point are equal and have a smooth transition.
    // ///
    // /// This method changes the underlying knot and element chain, by repeating some.
    // pub fn loop(self) -> BSplineDirector<K,E, Unknown, W>{
    //
    // }

    /// Set the elements of the bspline interpolation.
    pub fn elements<E>(self, elements: E) -> BSplineBuilder<Unknown, E, Unknown, WithoutWeight, M>
    where
        E: Chain,
    {
        BSplineBuilder {
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
    pub fn elements_with_weights<G>(
        self,
        signal: G,
    ) -> BSplineBuilder<Unknown, Weights<G>, Unknown, WithWeight, M>
    where
        G: Chain,
        G::Output: IntoWeight,
        <G::Output as IntoWeight>::Element:
            Mul<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
        <G::Output as IntoWeight>::Weight: Zero + Copy,
    {
        BSplineBuilder {
            inner: self.inner.and_then(|director| {
                director
                    .elements_with_weights(signal)
                    .map_err(|err| err.into())
            }),
        }
    }
}

impl<E, W> BSplineDirector<Unknown, E, Unknown, W, Open> {
    /// Set the knots of the interpolation.
    ///
    /// The degree of this bspline interplation is given by `knots.len() - elements.len() - 1`.
    ///
    /// # Errors
    ///
    /// Returns [`NotSorted`] if a knot is not greater or equal then the knot before him.
    /// Returns [`TooFewKnots`] if not at least 2 knots are given.
    /// Returns [`IncongruousElementsKnots`] if less knots than elements or more knots than twice as many elements are given.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using [`equidistant()`] instead.
    ///
    /// [`equidistant()`]: BSplineDirector::equidistant()
    /// [`NotSorted`]: super::error::BSplineError
    /// [`TooFewKnots`]: super::error::BSplineError
    /// [`IncongruousElementsKnots`]: super::error::BSplineError
    pub fn knots<K>(
        self,
        knots: K,
    ) -> Result<BSplineDirector<Sorted<K>, E, Unknown, W, Open>, BSplineError>
    where
        E: Chain,
        K: Chain,
        K::Output: PartialOrd,
    {
        if knots.len() < 2 {
            return Err(TooFewKnots::new(knots.len()).into());
        }
        if knots.len() < self.elements.len() {
            return Err(IncongruousElementsKnots::open(self.elements.len(), knots.len()).into());
        }
        if self.elements.len() <= knots.len() - self.elements.len() + 1 {
            return Err(IncongruousElementsKnots::open(self.elements.len(), knots.len()).into());
        }
        Ok(BSplineDirector {
            knots: Sorted::new(knots)?,
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        })
    }
}

impl<E, W> BSplineBuilder<Unknown, E, Unknown, W, Open> {
    /// Set the knots of the interpolation.
    ///
    /// The degree of this bspline interplation is given by `knots.len() - elements.len() - 1`.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using [`equidistant()`] instead.
    ///
    /// [`equidistant()`]: BSplineBuilder::equidistant()
    pub fn knots<K>(self, knots: K) -> BSplineBuilder<Sorted<K>, E, Unknown, W, Open>
    where
        E: Chain,
        K: Chain,
        K::Output: PartialOrd,
    {
        BSplineBuilder {
            inner: self.inner.and_then(|director| director.knots(knots)),
        }
    }
}

impl<E, W> BSplineDirector<Unknown, E, Unknown, W, Clamped> {
    /// Set the knots of the interpolation.
    ///
    /// The degree of this bspline interplation is given by `knots.len() - elements.len() - 1`.
    ///
    /// # Errors
    ///
    /// Returns [`NotSorted`] if a knot is not greater or equal then the knot before him.
    /// Returns [`TooFewKnots`] if not at least 2 knots are given.
    /// Returns [`IncongruousElementsKnots`] if less elements than knots are given.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using [`equidistant()`] instead.
    ///
    /// [`equidistant()`]: BSplineDirector::equidistant()
    /// [`NotSorted`]: super::BSplineError
    /// [`TooFewKnots`]: super::error::BSplineError
    /// [`IncongruousElementsKnots`]: super::error::BSplineError
    pub fn knots<K>(self, knots: K) -> Result<ClampedBSplineDirector<K, E, W>, BSplineError>
    where
        E: Chain,
        K: Chain,
        K::Output: PartialOrd,
    {
        if knots.len() < 2 {
            return Err(TooFewKnots::new(knots.len()).into());
        }
        if self.elements.len() < knots.len() {
            return Err(IncongruousElementsKnots::clamped(self.elements.len(), knots.len()).into());
        }
        let duplicate = self.elements.len() - knots.len(); //deg-1
        Ok(BSplineDirector {
            knots: BorderBuffer::new(Sorted::new(knots)?, duplicate),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        })
    }
}

impl<E, W> BSplineBuilder<Unknown, E, Unknown, W, Clamped> {
    /// Set the knots of the interpolation.
    ///
    /// The degree of this bspline interplation is given by `knots.len() - elements.len() - 1`.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using [`equidistant()`] instead.
    ///
    /// [`equidistant()`]: BSplineBuilder::equidistant()
    pub fn knots<K>(self, knots: K) -> ClampedBSplineBuilder<K, E, W>
    where
        E: Chain,
        K: Chain,
        K::Output: PartialOrd,
    {
        BSplineBuilder {
            inner: self.inner.and_then(|director| director.knots(knots)),
        }
    }
}

impl<E, W> BSplineDirector<Unknown, E, Unknown, W, Legacy> {
    /// Set the knots of the interpolation.
    ///
    /// The degree of this bspline interplation is given by `knots.len() - elements.len() - 1`.
    ///
    /// # Errors
    ///
    /// Returns [`NotSorted`] if a knot is not greater or equal then the knot before him.
    /// Returns [`TooFewKnots`] if not at least 2 knots are given.
    /// Returns [`IncongruousElementsKnots`] if less knots than elements + 2 are given.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using [`equidistant()`] instead.
    ///
    /// [`NotSorted`]: super::error::BSplineError
    /// [`TooFewKnots`]: super::error::BSplineError
    /// [`IncongruousElementsKnots`]: super::error::BSplineError
    /// [`equidistant()`]: BSplineDirector::equidistant()
    pub fn knots<K>(self, knots: K) -> Result<LegacyBSplineDirector<K, E, W>, BSplineError>
    where
        E: Chain,
        K: Chain,
        K::Output: PartialOrd,
    {
        if knots.len() < 4 {
            return Err(TooFewKnots::new(knots.len()).into());
        }
        if knots.len() <= self.elements.len() + 1 {
            return Err(IncongruousElementsKnots::legacy(self.elements.len(), knots.len()).into());
        }
        if self.elements.len() < knots.len() - self.elements.len() {
            return Err(IncongruousElementsKnots::legacy(self.elements.len(), knots.len()).into());
        }
        Ok(BSplineDirector {
            // unwrap is fine as we checked beforehand
            knots: BorderDeletion::new(Sorted::new(knots).unwrap())?,
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        })
    }
}

impl<E, W> BSplineBuilder<Unknown, E, Unknown, W, Legacy> {
    /// Set the knots of the interpolation.
    ///
    /// The degree of this bspline interplation is given by `knots.len() - elements.len() - 1`.
    ///
    /// # Performance
    ///
    /// If you have equidistant knots, near equidistant knots are you do not really care about
    /// knots, consider using [`equidistant()`] instead.
    ///
    /// [`equidistant()`]: BSplineBuilder::equidistant()
    pub fn knots<K>(self, knots: K) -> LegacyBSplineBuilder<K, E, W>
    where
        E: Chain,
        K: Chain,
        K::Output: PartialOrd,
    {
        BSplineBuilder {
            inner: self.inner.and_then(|director| director.knots(knots)),
        }
    }
}

impl<E, W, M> BSplineDirector<Unknown, E, Unknown, W, M> {
    /// Build an interpolation with equidistant knots.
    ///
    /// This method takes `R` as a generic parameter. `R` has to be the type you want the knots to be.
    /// Often this is just `f32` or `f64`.
    ///
    /// After this call, you also have to call either of [`degree()`] or [`quantity()`],
    /// which define the number of knots used.
    ///
    /// # Performance
    ///
    /// This may drastically increase performance, as one does not have to use binary search to find
    /// the relevant knots in an interpolation.
    ///
    /// [`degree()`]: BSplineDirector::domain()
    /// [`quantity()`]: BSplineDirector::normalized()
    pub fn equidistant<R>(self) -> BSplineDirector<Type<R>, E, Unknown, W, M> {
        BSplineDirector {
            knots: Type::new(),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        }
    }
}

impl<E, W, M> BSplineBuilder<Unknown, E, Unknown, W, M> {
    /// Build an interpolation with equidistant knots.
    ///
    /// This method takes `R` as a generic parameter. `R` has to be the type you want the knots to be.
    /// Often this is just `f32` or `f64`.
    ///
    /// After this call, you also have to call either of [`degree()`] or [`quantity()`],
    /// which define the number of knots used.
    ///
    /// # Performance
    ///
    /// This may drastically increase performance, as one does not have to use binary search to find
    /// the relevant knots in an interpolation.
    ///
    /// [`degree()`]: BSplineBuilder::domain()
    /// [`quantity()`]: BSplineBuilder::normalized()
    pub fn equidistant<R>(self) -> BSplineBuilder<Type<R>, E, Unknown, W, M> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.equidistant()),
        }
    }
}

impl<R, E, W> BSplineDirector<Type<R>, E, Unknown, W, Open>
where
    E: Chain,
{
    /// Set the degree of the curve.
    ///
    /// The degree of the curve has to be at least 1 and be less than the number of elements.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDegree`] if given degree is not at least 1.
    /// Returns [`IncongruousElementsDegree`] if given degree is not less than the amount of elements.
    ///
    /// [`InvalidDegree`]: super::error::BSplineError
    /// [`domain()`]: BSplineDirector::domain()
    /// [`normalized()`]: BSplineDirector::normalized()
    /// [`distance()`]: BSplineDirector::distance()
    pub fn degree(
        self,
        degree: usize,
    ) -> Result<BSplineDirector<UnknownDomain<R>, E, Unknown, W, Open>, BSplineError> {
        if degree < 1 {
            return Err(InvalidDegree::new(degree).into());
        }
        if self.elements.len() <= degree {
            return Err(IncongruousElementsDegree::open(self.elements.len(), degree).into());
        }
        Ok(BSplineDirector {
            knots: UnknownDomain::new(self.elements.len() - 1 + degree, degree),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        })
    }

    /// Set the number of knots.
    ///
    /// For open curves, the number of knots has to be bigger then the number of elements.
    /// For closed curves, the number of knots has to be at most as big as the number of elements.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Errors
    ///
    /// Returns [`TooFewKnots`] if not at least 2 knots are given.
    /// Returns [`IncongruousElementsKnots`] if less knots than elements or more knots than double the amount of elements are given.
    ///
    /// [`TooFewKnots`]: super::error::BSplineError
    /// [`IncongruousElementsKnots`]: super::error::BSplineError
    /// [`domain()`]: BSplineDirector::domain()
    /// [`normalized()`]: BSplineDirector::normalized()
    /// [`distance()`]: BSplineDirector::distance()
    pub fn quantity(
        self,
        quantity: usize,
    ) -> Result<BSplineDirector<UnknownDomain<R>, E, Unknown, W, Open>, BSplineError> {
        if quantity < 2 {
            return Err(TooFewKnots::new(quantity).into());
        }
        if quantity < self.elements.len() {
            return Err(IncongruousElementsKnots::legacy(self.elements.len(), quantity).into());
        }
        if self.elements.len() <= quantity - self.elements.len() + 1 {
            return Err(IncongruousElementsKnots::open(self.elements.len(), quantity).into());
        }
        Ok(BSplineDirector {
            knots: UnknownDomain::new(quantity, quantity - self.elements.len() + 1),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        })
    }
}

impl<R, E, W> BSplineBuilder<Type<R>, E, Unknown, W, Open>
where
    E: Chain,
{
    /// Set the degree of the curve.
    ///
    /// The degree of the curve has to be at least 1 and be less than the number of elements.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements is zero.
    ///
    /// [`domain()`]: BSplineBuilder::domain()
    /// [`normalized()`]: BSplineBuilder::normalized()
    /// [`distance()`]: BSplineBuilder::distance()
    pub fn degree(self, degree: usize) -> BSplineBuilder<UnknownDomain<R>, E, Unknown, W, Open> {
        BSplineBuilder {
            inner: self.inner.and_then(|director| director.degree(degree)),
        }
    }

    /// Set the number of knots.
    ///
    /// For open curves, the number of knots has to be bigger then the number of elements.
    /// For closed curves, the number of knots has to be at most as big as the number of elements.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Panics
    ///
    /// Panics if given quantity is less than the number of elements.
    /// May also panic if the number of elements is zero.
    ///
    /// [`domain()`]: BSplineBuilder::domain()
    /// [`normalized()`]: BSplineBuilder::normalized()
    /// [`distance()`]: BSplineBuilder::distance()
    pub fn quantity(
        self,
        quantity: usize,
    ) -> BSplineBuilder<UnknownDomain<R>, E, Unknown, W, Open> {
        BSplineBuilder {
            inner: self.inner.and_then(|director| director.quantity(quantity)),
        }
    }
}

impl<R, E, W> BSplineDirector<Type<R>, E, Unknown, W, Clamped>
where
    E: Chain,
{
    /// Set the degree of the curve.
    ///
    /// The degree of the curve has to be at least 1 and be less than the number of elements.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    ///
    /// # Errors
    ///
    /// Returns [`InvalidDegree`] if given degree is 0.
    /// Returns [`IncongruousElementsDegree`] if degree is not less than the number of elements.
    ///
    /// [`InvalidDegree`]: super::error::BSplineError
    /// [`IncongruousElementsDegree`]: super::error::BSplineError
    /// [`domain()`]: BSplineDirector::domain()
    /// [`normalized()`]: BSplineDirector::normalized()
    /// [`distance()`]: BSplineDirector::distance()
    /// [`domain()`]: BSplineDirector::domain()
    pub fn degree(
        self,
        degree: usize,
    ) -> Result<BSplineDirector<UnknownDomain<R>, E, Unknown, W, Clamped>, BSplineError> {
        if degree < 1 {
            return Err(InvalidDegree::new(degree).into());
        }
        if self.elements.len() <= degree {
            return Err(IncongruousElementsDegree::clamped(self.elements.len(), degree).into());
        }
        Ok(BSplineDirector {
            knots: UnknownDomain::new(self.elements.len() - degree + 1, degree),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        })
    }

    /// Set the number of knots.
    ///
    /// For open curves, the number of knots has to be bigger then the number of elements.
    /// For closed curves, the number of knots has to be at most as big as the number of elements.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Errors
    ///
    /// Returns [`TooFewKnots`] if given quantity is less than 2.
    /// Returns [`IncongruousElementsKnots`] if less elements than knots are given.
    ///
    /// [`TooFewKnots`]: super::error::BSplineError
    /// [`IncongruousElementsKnots`]: super::error::BSplineError
    /// [`domain()`]: BSplineDirector::domain()
    /// [`normalized()`]: BSplineDirector::normalized()
    /// [`distance()`]: BSplineDirector::distance()
    pub fn quantity(
        self,
        quantity: usize,
    ) -> Result<BSplineDirector<UnknownDomain<R>, E, Unknown, W, Clamped>, BSplineError> {
        if quantity < 2 {
            return Err(TooFewKnots::new(quantity).into());
        }
        if self.elements.len() < quantity {
            return Err(IncongruousElementsKnots::clamped(self.elements.len(), quantity).into());
        }
        Ok(BSplineDirector {
            knots: UnknownDomain::new(quantity, self.elements.len() - quantity + 1),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        })
    }
}

impl<R, E, W> BSplineBuilder<Type<R>, E, Unknown, W, Clamped>
where
    E: Chain,
{
    /// Set the degree of the curve.
    ///
    /// The degree of the curve has to be at least 1 and be less than the number of elements.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements is zero.
    ///
    /// [`domain()`]: BSplineBuilder::domain()
    /// [`normalized()`]: BSplineBuilder::normalized()
    /// [`distance()`]: BSplineBuilder::distance()
    pub fn degree(self, degree: usize) -> BSplineBuilder<UnknownDomain<R>, E, Unknown, W, Clamped> {
        BSplineBuilder {
            inner: self.inner.and_then(|director| director.degree(degree)),
        }
    }

    /// Set the number of knots.
    ///
    /// For open curves, the number of knots has to be bigger then the number of elements.
    /// For closed curves, the number of knots has to be at most as big as the number of elements.
    ///
    /// After this call, you also have to call either of
    /// - [`domain()`],
    /// - [`normalized()`] or
    /// - [`distance()`],
    ///
    /// which all define the domain of the interpolation and the spacing of the knots.
    ///
    /// # Panics
    ///
    /// Panics if given quantity is less than the number of elements.
    /// May also panic if the number of elements is zero.
    ///
    /// [`domain()`]: BSplineBuilder::domain()
    /// [`normalized()`]: BSplineBuilder::normalized()
    /// [`distance()`]: BSplineBuilder::distance()
    pub fn quantity(
        self,
        quantity: usize,
    ) -> BSplineBuilder<UnknownDomain<R>, E, Unknown, W, Clamped> {
        BSplineBuilder {
            inner: self.inner.and_then(|director| director.quantity(quantity)),
        }
    }
}

impl<R, E, W> BSplineDirector<UnknownDomain<R>, E, Unknown, W, Open>
where
    E: Chain,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(self, start: R, end: R) -> BSplineDirector<Equidistant<R>, E, Unknown, W, Open> {
        BSplineDirector {
            knots: Equidistant::new(self.knots.len(), start, end),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(self) -> BSplineDirector<Equidistant<R>, E, Unknown, W, Open> {
        BSplineDirector {
            knots: Equidistant::normalized(self.knots.len()),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots.
    pub fn distance(
        self,
        start: R,
        step: R,
    ) -> BSplineDirector<Equidistant<R>, E, Unknown, W, Open> {
        BSplineDirector {
            knots: Equidistant::step(self.knots.len(), start, step),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        }
    }
}

impl<R, E, W> BSplineBuilder<UnknownDomain<R>, E, Unknown, W, Open>
where
    E: Chain,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(self, start: R, end: R) -> BSplineBuilder<Equidistant<R>, E, Unknown, W, Open> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.domain(start, end)),
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(self) -> BSplineBuilder<Equidistant<R>, E, Unknown, W, Open> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.normalized()),
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots.
    pub fn distance(
        self,
        start: R,
        step: R,
    ) -> BSplineBuilder<Equidistant<R>, E, Unknown, W, Open> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.distance(start, step)),
        }
    }
}

impl<R, E, W> BSplineDirector<UnknownDomain<R>, E, Unknown, W, Clamped>
where
    E: Chain,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(
        self,
        start: R,
        end: R,
    ) -> BSplineDirector<BorderBuffer<Equidistant<R>>, E, Unknown, W, Clamped> {
        BSplineDirector {
            knots: BorderBuffer::new(
                Equidistant::new(self.knots.len(), start, end),
                self.knots.deg() - 1,
            ),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(
        self,
    ) -> BSplineDirector<BorderBuffer<Equidistant<R>>, E, Unknown, W, Clamped> {
        BSplineDirector {
            knots: BorderBuffer::new(
                Equidistant::normalized(self.knots.len()),
                self.knots.deg() - 1,
            ),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots.
    pub fn distance(
        self,
        start: R,
        step: R,
    ) -> BSplineDirector<BorderBuffer<Equidistant<R>>, E, Unknown, W, Clamped> {
        BSplineDirector {
            knots: BorderBuffer::new(
                Equidistant::step(self.knots.len(), start, step),
                self.knots.deg() - 1,
            ),
            elements: self.elements,
            space: self.space,
            _phantoms: self._phantoms,
        }
    }
}

impl<R, E, W> BSplineBuilder<UnknownDomain<R>, E, Unknown, W, Clamped>
where
    E: Chain,
    R: Real + FromPrimitive,
{
    /// Set the domain of the interpolation.
    pub fn domain(
        self,
        start: R,
        end: R,
    ) -> BSplineBuilder<BorderBuffer<Equidistant<R>>, E, Unknown, W, Clamped> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.domain(start, end)),
        }
    }

    /// Set the domain of the interpolation to be [0.0,1.0].
    pub fn normalized(
        self,
    ) -> BSplineBuilder<BorderBuffer<Equidistant<R>>, E, Unknown, W, Clamped> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.normalized()),
        }
    }
    /// Set the domain of the interpolation by defining the distance between the knots.
    pub fn distance(
        self,
        start: R,
        step: R,
    ) -> BSplineBuilder<BorderBuffer<Equidistant<R>>, E, Unknown, W, Clamped> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.distance(start, step)),
        }
    }
}

impl<K, E, W, M> BSplineDirector<K, E, Unknown, W, M>
where
    E: Chain,
    K: Chain,
{
    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder to use a vector as workspace,
    /// such you don't need to know the degree of the bezier curve at compile-time,
    /// but for every generation of a value an allocation of memory will be necessary.
    ///
    /// If the degree of the bezier curve is known at compile-time, consider using [`constant()`] instead.
    ///
    /// [`constant()`]: BSplineDirector::constant()
    #[cfg(feature = "std")]
    pub fn dynamic(self) -> BSplineDirector<K, E, DynSpace<E::Output>, W, M> {
        BSplineDirector {
            space: DynSpace::new(self.knots.len() - self.elements.len() + 2),
            knots: self.knots,
            elements: self.elements,
            _phantoms: self._phantoms,
        }
    }

    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder the size of the workspace needed such that no memory allocations are necessary
    /// when interpolating.
    ///
    /// The size needed is `degree + 1` and/or `knots.len() - elements.len()`.
    #[allow(clippy::type_complexity)]
    pub fn constant<const N: usize>(
        self,
    ) -> Result<BSplineDirector<K, E, ConstSpace<E::Output, N>, W, M>, TooSmallWorkspace> {
        // This calculation won't panic as we checked before if the degree is not strictly positive.
        if N <= self.knots.len() - self.elements.len() + 1 {
            return Err(TooSmallWorkspace::new(
                N,
                self.knots.len() - self.elements.len() + 1,
            ));
        }
        Ok(BSplineDirector {
            knots: self.knots,
            space: ConstSpace::new(),
            elements: self.elements,
            _phantoms: self._phantoms,
        })
    }

    /// Set the workspace which the interpolation uses.
    ///
    /// This method should be applied if one don't want to or can't use `Vector`.
    ///
    /// If the degree of the bezier curve is known at compile-time, consider using [`constant()`] instead.
    /// Otherwise without std support, one has to set a specific object implementing the [`Space`] trait.
    ///
    /// [`constant()`]: BSplineDirector::constant()
    /// [`Space`]: crate::base::Space
    pub fn workspace<S>(self, space: S) -> Result<BSplineDirector<K, E, S, W, M>, TooSmallWorkspace>
    where
        S: Space<E::Output>,
    {
        if space.len() <= self.knots.len() - self.elements.len() + 1 {
            return Err(TooSmallWorkspace::new(
                space.len(),
                self.knots.len() - self.elements.len() + 1,
            ));
        }
        Ok(BSplineDirector {
            knots: self.knots,
            space,
            elements: self.elements,
            _phantoms: self._phantoms,
        })
    }
}

impl<K, E, W, M> BSplineBuilder<K, E, Unknown, W, M>
where
    E: Chain,
    K: Chain,
{
    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder to use a vector as workspace,
    /// such you don't need to know the degree of the bezier curve at compile-time,
    /// but for every generation of a value an allocation of memory will be necessary.
    ///
    /// If the degree of the bezier curve is known at compile-time, consider using [`constant()`] instead.
    ///
    /// [`constant()`]: BSplineBuilder::constant()
    #[cfg(feature = "std")]
    pub fn dynamic(self) -> BSplineBuilder<K, E, DynSpace<E::Output>, W, M> {
        BSplineBuilder {
            inner: self.inner.map(|director| director.dynamic()),
        }
    }

    /// Set the workspace which the interpolation uses.
    ///
    /// Tells the builder the size of the workspace needed such that no memory allocations are necessary
    /// when interpolating.
    ///
    /// The size needed is `degree + 1` and/or `knots.len() - elements.len()`.
    pub fn constant<const N: usize>(self) -> BSplineBuilder<K, E, ConstSpace<E::Output, N>, W, M> {
        BSplineBuilder {
            inner: self
                .inner
                .and_then(|director| director.constant().map_err(|err| err.into())),
        }
    }

    /// Set the workspace which the interpolation uses.
    ///
    /// This method should be applied if one don't want to or can't use `Vector`.
    ///
    /// If the degree of the bezier curve is known at compile-time, consider using [`constant()`] instead.
    /// Otherwise without std support, one has to set a specific object implementing the [`Space`] trait.
    ///
    /// [`constant()`]: BSplineBuilder::constant()
    /// [`Space`]: crate::base::Space
    pub fn workspace<S>(self, space: S) -> BSplineBuilder<K, E, S, W, M>
    where
        S: Space<E::Output>,
    {
        BSplineBuilder {
            inner: self
                .inner
                .and_then(|director| director.workspace(space).map_err(|err| err.into())),
        }
    }
}

impl<K, E, S, M> BSplineDirector<K, E, S, WithoutWeight, M>
where
    K: SortedChain,
    E: Chain,
    E::Output: Merge<K::Output> + Copy,
    S: Space<E::Output>,
{
    /// Build a bezier interpolation.
    ///
    /// # Errors
    ///
    /// [`TooFewElements`] if there are less than two elements.
    /// [`InvalidDegree`] if degree is not at least 1 and at most the number of elements - 1.
    /// [`TooSmallWorkspace`] if the workspace is not bigger than the degree of the curve.
    ///
    /// [`TooFewElements`]: super::BSplineError
    /// [`InvalidDegree`]: super::BSplineError
    /// [`TooSmallWorkspace`]: super::BSplineError
    pub fn build(self) -> BSpline<K, E, S> {
        BSpline::new_unchecked(self.elements, self.knots, self.space)
    }
}

impl<K, E, S, M> BSplineBuilder<K, E, S, WithoutWeight, M>
where
    K: SortedChain,
    E: Chain,
    E::Output: Merge<K::Output> + Copy,
    S: Space<E::Output>,
{
    /// Build a bezier interpolation.
    ///
    /// # Errors
    ///
    /// [`TooFewElements`] if there are less than two elements or less than four elements in legacy mode.
    /// [`InvalidDegree`] if degree is not at least 1 and at most the number of elements - 1.
    /// [`TooSmallWorkspace`] if the workspace is not bigger than the degree of the curve.
    /// [`NotSorted`] if the knots given in the method [`knots()`] were not sorted.
    ///
    /// [`TooFewElements`]: super::BSplineError
    /// [`InvalidDegree`]: super::BSplineError
    /// [`TooSmallWorkspace`]: super::BSplineError
    /// [`NotSorted`]: super::BSplineError
    /// [`knots()`]: BSplineBuilder::knots()
    pub fn build(self) -> Result<BSpline<K, E, S>, BSplineError> {
        match self.inner {
            Err(err) => Err(err),
            Ok(director) => Ok(director.build()),
        }
    }
}

impl<K, G, S, M> BSplineDirector<K, Weights<G>, S, WithWeight, M>
where
    G: Chain,
    G::Output: IntoWeight,
    K: SortedChain,
    S: Space<Homogeneous<<G::Output as IntoWeight>::Element, <G::Output as IntoWeight>::Weight>>,
    <Weights<G> as Signal<usize>>::Output: Merge<K::Output> + Copy,
    <G::Output as IntoWeight>::Element:
        Div<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
{
    /// Build a bezier interpolation.
    ///
    /// # Errors
    ///
    /// [`TooFewElements`] if there are less than two elements.
    /// [`InvalidDegree`] if degree is not at least 1 and at most the number of elements - 1.
    /// [`TooSmallWorkspace`] if the workspace is not bigger than the degree of the curve.
    ///
    /// [`TooFewElements`]: super::BSplineError
    /// [`InvalidDegree`]: super::BSplineError
    /// [`TooSmallWorkspace`]: super::BSplineError
    pub fn build(self) -> WeightedBSpline<K, G, S> {
        Weighted::new(BSpline::new_unchecked(
            self.elements,
            self.knots,
            self.space,
        ))
    }
}

impl<K, G, S, M> BSplineBuilder<K, Weights<G>, S, WithWeight, M>
where
    G: Chain,
    G::Output: IntoWeight,
    K: SortedChain,
    S: Space<Homogeneous<<G::Output as IntoWeight>::Element, <G::Output as IntoWeight>::Weight>>,
    <Weights<G> as Signal<usize>>::Output: Merge<K::Output> + Copy,
    <G::Output as IntoWeight>::Element:
        Div<<G::Output as IntoWeight>::Weight, Output = <G::Output as IntoWeight>::Element>,
{
    /// Build a bezier interpolation.
    ///
    /// # Errors
    ///
    /// [`TooFewElements`] if there are less than two elements or less than four elements in legacy mode.
    /// [`InvalidDegree`] if degree is not at least 1 and at most the number of elements - 1.
    /// [`TooSmallWorkspace`] if the workspace is not bigger than the degree of the curve.
    /// [`NotSorted`] if the knots given in the method [`knots()`] were not sorted.
    ///
    /// [`TooFewElements`]: super::BSplineError
    /// [`InvalidDegree`]: super::BSplineError
    /// [`TooSmallWorkspace`]: super::BSplineError
    /// [`NotSorted`]: super::BSplineError
    /// [`knots()`]: BSplineBuilder::knots()
    pub fn build(self) -> Result<WeightedBSpline<K, G, S>, BSplineError> {
        match self.inner {
            Err(err) => Err(err),
            Ok(director) => Ok(director.build()),
        }
    }
}

/// Type alias for weighted bsplines.
type WeightedBSpline<K, G, S> = Weighted<BSpline<K, Weights<G>, S>>;
/// Type alias for ClampedBuilder
type ClampedBSplineBuilder<K, E, W> =
    BSplineBuilder<BorderBuffer<Sorted<K>>, E, Unknown, W, Clamped>;
/// Type alias for ClampedDirector
type ClampedBSplineDirector<K, E, W> =
    BSplineDirector<BorderBuffer<Sorted<K>>, E, Unknown, W, Clamped>;
///Type alias for LegacyBuilder
type LegacyBSplineBuilder<K, E, W> =
    BSplineBuilder<BorderDeletion<Sorted<K>>, E, Unknown, W, Legacy>;
///Type alias for LegacyDirector
type LegacyBSplineDirector<K, E, W> =
    BSplineDirector<BorderDeletion<Sorted<K>>, E, Unknown, W, Legacy>;

#[cfg(test)]
mod test {
    use super::BSplineBuilder;
    // Homogeneous for creating Homogeneous, Signal for using .stack()
    use crate::{Curve, Signal, bspline::BSplineDirector, weights::Homogeneous};

    #[test]
    fn degenerate_creations() {
        let empty: [f64; 0] = [];
        assert!(
            BSplineBuilder::new()
                .elements(empty)
                .knots(empty)
                .constant::<1>()
                .build()
                .is_err()
        );
        assert!(
            BSplineBuilder::new()
                .elements([1.0])
                .knots([1.0])
                .constant::<2>()
                .build()
                .is_err()
        );
    }

    #[test]
    fn mode_equality() {
        let elements = [1.0, 3.0, 7.0];
        let open = BSplineBuilder::new()
            .elements(elements)
            .knots([0.0, 0.0, 1.0, 1.0])
            .constant::<3>()
            .build()
            .unwrap();
        let clamped = BSplineBuilder::new()
            .clamped()
            .elements(elements)
            .knots([0.0, 1.0])
            .constant::<3>()
            .build()
            .unwrap();
        let legacy = BSplineBuilder::new()
            .legacy()
            .elements(elements)
            .knots([0.0, 0.0, 0.0, 1.0, 1.0, 1.0])
            .constant::<3>()
            .build()
            .unwrap();
        for (a, b, c) in open
            .take(10)
            .zip(clamped.take(10))
            .zip(legacy.take(10))
            .map(|((a, b), c)| (a, b, c))
        {
            assert_f64_near!(a, b);
            assert_f64_near!(b, c);
        }
    }

    #[test]
    fn elements_with_weights() {
        BSplineBuilder::new()
            .elements_with_weights([(1.0, 1.0), (2.0, 2.0), (3.0, 0.0)])
            .equidistant::<f64>()
            .degree(2)
            .domain(0.0, 5.0)
            .constant::<3>()
            .build()
            .unwrap();
        BSplineBuilder::new()
            .elements_with_weights([1.0, 2.0, 3.0].stack([1.0, 2.0, 0.0]))
            .equidistant::<f64>()
            .degree(1)
            .normalized()
            .constant::<2>()
            .build()
            .unwrap();
        BSplineBuilder::new()
            .elements_with_weights([
                Homogeneous::new(1.0),
                Homogeneous::weighted_unchecked(2.0, 2.0),
                Homogeneous::infinity(3.0),
            ])
            .knots([1.0, 2.0, 3.0])
            .constant::<2>()
            .build()
            .unwrap();
        BSplineBuilder::new()
            .elements([0.1, 0.2, 0.3])
            .equidistant::<f64>()
            .degree(1)
            .normalized()
            .constant::<2>()
            .build()
            .unwrap();
    }

    #[test]
    fn clamped_errors() {
        // too few elements
        assert!(BSplineDirector::new().clamped().elements([0.0]).is_err());

        // too few knots
        assert!(
            BSplineDirector::new()
                .clamped()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .knots([0.0])
                .is_err()
        );

        assert!(
            BSplineDirector::new()
                .clamped()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .equidistant::<f32>()
                .quantity(1)
                .is_err()
        );

        // invalid degree
        assert!(
            BSplineDirector::new()
                .clamped()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .equidistant::<f32>()
                .degree(0)
                .is_err()
        );

        // too small of a workspace
        assert!(
            BSplineDirector::new()
                .clamped()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .equidistant::<f32>()
                .degree(2)
                .unwrap()
                .domain(0.0, 1.0)
                .constant::<2>()
                .is_err()
        );

        // incongruous
        assert!(
            BSplineDirector::new()
                .clamped()
                .elements([0.0, 1.0, 2.0])
                .unwrap()
                .equidistant::<f32>()
                .degree(3)
                .is_err()
        );

        assert!(
            BSplineDirector::new()
                .clamped()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .equidistant::<f32>()
                .degree(3)
                .is_ok()
        );

        assert!(
            BSplineDirector::new()
                .clamped()
                .elements([0.0, 1.0, 2.0])
                .unwrap()
                .equidistant::<f32>()
                .quantity(4)
                .is_err()
        );

        assert!(
            BSplineDirector::new()
                .clamped()
                .elements([0.0, 1.0, 2.0])
                .unwrap()
                .equidistant::<f32>()
                .quantity(3)
                .is_ok()
        );
    }

    #[test]
    fn open_errors() {
        // too few elements
        assert!(BSplineDirector::new().open().elements([0.0]).is_err());

        // too few knots
        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .knots([0.0])
                .is_err()
        );

        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .equidistant::<f32>()
                .quantity(1)
                .is_err()
        );

        // invalid degree
        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .equidistant::<f32>()
                .degree(0)
                .is_err()
        );

        // too small of a workspace
        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .equidistant::<f32>()
                .degree(2)
                .unwrap()
                .domain(0.0, 1.0)
                .constant::<2>()
                .is_err()
        );

        // incongruous knots
        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0, 2.0])
                .unwrap()
                .equidistant::<f32>()
                .quantity(2)
                .is_err()
        );

        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0, 2.0])
                .unwrap()
                .equidistant::<f32>()
                .quantity(3)
                .is_ok()
        );

        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0, 2.0])
                .unwrap()
                .equidistant::<f32>()
                .quantity(4)
                .is_ok()
        );

        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0, 2.0])
                .unwrap()
                .equidistant::<f32>()
                .quantity(5)
                .is_err()
        );

        // incongruous degree
        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0])
                .unwrap()
                .equidistant::<f32>()
                .degree(1)
                .is_ok()
        );

        assert!(
            BSplineDirector::new()
                .open()
                .elements([0.0, 1.0])
                .unwrap()
                .equidistant::<f32>()
                .degree(2)
                .is_err()
        );
    }

    #[test]
    fn legacy_errors() {
        // too few elements
        assert!(BSplineDirector::new().legacy().elements([0.0]).is_err());

        // too few knots
        assert!(
            BSplineDirector::new()
                .legacy()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .knots([0.0, 1.0, 2.0])
                .is_err()
        );

        // too small of a workspace
        assert!(
            BSplineDirector::new()
                .legacy()
                .elements([0.0, 1.0, 2.0, 3.0])
                .unwrap()
                .knots([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0])
                .unwrap()
                .constant::<2>()
                .is_err()
        );

        // incongruous knots
        assert!(
            BSplineDirector::new()
                .legacy()
                .elements([0.0, 1.0, 3.0])
                .unwrap()
                .knots([0.0, 1.0, 2.0, 3.0])
                .is_err()
        );

        assert!(
            BSplineDirector::new()
                .legacy()
                .elements([0.0, 1.0, 3.0])
                .unwrap()
                .knots([0.0, 1.0, 2.0, 3.0, 4.0])
                .is_ok()
        );

        assert!(
            BSplineDirector::new()
                .legacy()
                .elements([0.0, 1.0, 3.0])
                .unwrap()
                .knots([0.0, 1.0, 2.0, 3.0, 4.0, 5.0])
                .is_ok()
        );

        assert!(
            BSplineDirector::new()
                .legacy()
                .elements([0.0, 1.0, 3.0])
                .unwrap()
                .knots([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
                .is_err()
        );
    }
}
