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
