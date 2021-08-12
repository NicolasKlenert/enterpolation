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

#![ cfg_attr( not(feature = "std"), no_std ) ]

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

#[cfg(not(any(feature = "std", feature = "libm")))]
compile_error!(
    "The enterpolation crate needs a library for floats. Please enable either \"std\" or \"libm\" as a feature."
);

#[cfg(feature = "linear")]
pub mod linear;
#[cfg(feature = "bezier")]
pub mod bezier;
#[cfg(feature = "bspline")]
pub mod bspline;
pub mod weights;
pub mod utils;
pub mod easing;

mod base;
mod builder;

pub use topology_traits::Merge;

#[cfg(feature = "std")]
pub use base::DynSpace;
pub use base::{Generator, Interpolation, Curve, Extract, Stepper, Space, ConstSpace,
    DiscreteGenerator, ConstDiscreteGenerator, Equidistant, ConstEquidistant,
    Sorted, SortedGenerator, NotSorted, TransformInput, Chain, Stack, Slice, Repeat, Wrap};
pub use easing::{Easing, Identity};
// pub use weights::{Homogeneous, Weighted, Weights, IntoWeight};
