//! At some cases homogeneous points are needed to describe a specific curve or surface.
//! Either one wants to use a weighted formula of some curve or one may want to describe a point to
//! be at infinity. In both cases one wants to use rational curves. This module
//! gives you a wrapper at hand which transforms any interplation into a rational interpolation.

use core::ops::{Add, Div, Mul, Sub};
use num_traits::identities::{One, Zero};

/// Wrapper for elements to achieve weighted and rational curves.
///
/// This wrapper allows for Homogeneous Coordinates.
#[derive(PartialEq, Clone, Copy, Hash, Default, Debug)]
pub struct Homogeneous<E, R> {
    element: E,
    rational: R,
}

impl<E, R> Homogeneous<E, R>
where
    R: One,
{
    /// Lift your element to create a homogeneous coordinate.
    pub fn new(element: E) -> Self {
        Homogeneous {
            element,
            rational: R::one(),
        }
    }
}

impl<E, R> Homogeneous<E, R>
where
    R: Zero,
{
    /// Create a homogeneous coordinate which lies at infinity in the given direction.
    pub fn infinity(direction: E) -> Self {
        Homogeneous {
            element: direction,
            rational: R::zero(),
        }
    }
}

impl<E, R> Homogeneous<E, R>
where
    R: Zero + PartialEq,
{
    /// Returns true if value lies at infinity.
    pub fn is_infinite(&self) -> bool {
        self.rational == R::zero()
    }
}

impl<E, R> Homogeneous<E, R>
where
    E: Copy,
{
    /// Return direction of the coordinate.
    pub fn direction(&self) -> E {
        self.element
    }
}

impl<E, R> Homogeneous<E, R>
where
    E: Mul<R, Output = E>,
    R: Zero + Copy,
{
    /// Create a homogeneous coordinate with the specified weight as long as the given weight is not zero.
    ///
    /// The weight should not be zero. If you want to represent a point at infinity, use
    /// `infinity` instead.
    pub fn weighted(element: E, weight: R) -> Option<Self> {
        if weight.is_zero() {
            return None;
        }
        Some(Self::weighted_unchecked(element, weight))
    }

    /// Create a homogeneous coordinate with the specified weight as long as the given weight is not zero.
    /// Otherwise it will be infinite with the given element as direction.
    pub fn weighted_or_infinite(element: E, weight: R) -> Self {
        if weight.is_zero() {
            return Self::infinity(element);
        }
        Self::weighted_unchecked(element, weight)
    }
}

impl<E, R> Homogeneous<E, R>
where
    E: Mul<R, Output = E>,
    R: One + Zero + Copy,
{
    /// Create a homogeneous coordinate with the specified weight as long as the given weight is not zero.
    /// Otherwise it falls back to the weight of one.
    pub fn weighted_or_one(element: E, weight: R) -> Self {
        if weight.is_zero() {
            return Self::new(element);
        }
        Self::weighted_unchecked(element, weight)
    }
}

impl<E, R> Homogeneous<E, R>
where
    E: Mul<R, Output = E>,
    R: Copy,
{
    /// Create a homogeneous coordinate with the specified weight
    ///
    /// The weight should not be zero. If you want to represent a point at infinity, use
    /// `infinity` instead.
    ///
    /// # Requirements
    ///
    /// Given weight shall *not* be zero.
    pub fn weighted_unchecked(element: E, weight: R) -> Self {
        Homogeneous {
            element: element * weight,
            rational: weight,
        }
    }
}

impl<E, R> Homogeneous<E, R>
where
    E: Div<R, Output = E>,
{
    /// Project the homogenous coordinate back to the element space.
    ///
    /// The created element may not be a real number, but inf, NaN or such.
    pub fn project(self) -> E {
        self.element / self.rational
    }
}

impl<E, R> Add for Homogeneous<E, R>
where
    E: Add<Output = E>,
    R: Add<Output = R>,
{
    type Output = Homogeneous<E, R>;
    fn add(self, rhs: Homogeneous<E, R>) -> Self::Output {
        Homogeneous {
            element: self.element + rhs.element,
            rational: self.rational + rhs.rational,
        }
    }
}

impl<E, R> Sub for Homogeneous<E, R>
where
    E: Sub<Output = E>,
    R: Sub<Output = R>,
{
    type Output = Homogeneous<E, R>;
    fn sub(self, rhs: Homogeneous<E, R>) -> Self::Output {
        Homogeneous {
            element: self.element - rhs.element,
            rational: self.rational - rhs.rational,
        }
    }
}

impl<E, R> Mul for Homogeneous<E, R>
where
    E: Mul<Output = E>,
    R: Mul<Output = R>,
{
    type Output = Homogeneous<E, R>;
    fn mul(self, rhs: Homogeneous<E, R>) -> Self::Output {
        Homogeneous {
            element: self.element * rhs.element,
            rational: self.rational * rhs.rational,
        }
    }
}

impl<E, R> Div for Homogeneous<E, R>
where
    E: Div<Output = E>,
    R: Div<Output = R>,
{
    type Output = Homogeneous<E, R>;
    fn div(self, rhs: Homogeneous<E, R>) -> Self::Output {
        Homogeneous {
            element: self.element / rhs.element,
            rational: self.rational / rhs.rational,
        }
    }
}

impl<E, R> Mul<R> for Homogeneous<E, R>
where
    E: Mul<R, Output = E>,
    R: Mul<Output = R> + Copy,
{
    type Output = Homogeneous<E, R>;
    fn mul(self, rhs: R) -> Self::Output {
        Homogeneous {
            element: self.element * rhs,
            rational: self.rational * rhs,
        }
    }
}

impl<E, R> Div<R> for Homogeneous<E, R>
where
    E: Div<R, Output = E>,
    R: Div<Output = R> + Copy,
{
    type Output = Homogeneous<E, R>;
    fn div(self, rhs: R) -> Self::Output {
        Homogeneous {
            element: self.element / rhs,
            rational: self.rational / rhs,
        }
    }
}
