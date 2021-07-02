//! This crate provides a myriad of different interpolation, extrapolation and animation methods.
//! Most notably it tries to be generic and modular. If instances of your type act somewhat like
//! a vector space, this crate will be able to interpolate, extrapolate and animate them.
//! TODO: describe more

//TODO: Also our min and max_index of our linear interpolation does NOT clamp the values together...array out of bounds!
//TODO: SO upper_border can be broken somehow! Look into it!

//TODO: all interpolations should have as knots field not just K but Sorted(NonEmpty(K))
//TODO: all interpolations should have as elements field not just E but NonEmpty(E)
//TODO: we want to achieve many different creation options such that a builder will be necessary
//TODO: for now, create a builder for each different interpolation!
//TODO: Afterwards delete the implementation of SortedList for array and vec
//TODO: and add NonEmpty as super trait for SortedList!

#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

#[macro_use]
extern crate assert_float_eq;

pub mod linear;
pub mod bezier;
pub mod bspline;
pub mod utils;
pub mod weights;

// mod real;
// mod never;
mod base;
mod builder;

pub use base::{Generator, Interpolation, Curve, Extract, Stepper, Space, ConstSpace, DynSpace,
    DiscreteGenerator, ConstDiscreteGenerator, Equidistant, ConstEquidistant,
    Sorted, SortedGenerator, NotSorted, TransformInput, Chain, Stack, Slice, Repeat, Wrap, BorderBuffer};
// pub use weights::{Homogeneous, Weighted, Weights, IntoWeight};

use core::ops::{Add,Mul};
use num_traits::real::Real;

/// The merge trait is used to merge two elements together.
///
/// Often this is a linear interpolation between two elements.
/// In the case of Quaternions it is a spherical linear interpolation.
///
/// A default implementation of this trait is provided for all `E` that
/// are `Add<Output = E> + Mul<T, Output = E> + Copy` as these
/// operations let us assume that the elements live in a vector-like space.
///
/// Optimally you would never have to implement this trait.
pub trait Merge<T = f64> {
    /// Merge between `self` and `other` using `factor`.
    ///
    /// Merging `self` with a factor of `Zero` should return a copy of `self`.
    /// Merging `other` with a factor of `One` should return a copy of `other`.
    /// It is assumed that the factor decides how similar the result will be to either
    /// `self` or `other`.
    fn merge(self,other: Self, factor: T) -> Self;
}

impl<E,T> Merge<T> for E
where
    E: Add<Output = E> + Mul<T,Output = E> + Copy,
    T: Real,
{
    fn merge(self, other: Self, factor: T) -> E {
        self * (T::one() - factor) + other * factor
    }
}
