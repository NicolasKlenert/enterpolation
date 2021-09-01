#![doc = include_str!("../README.md")]
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
pub use base::{Generator, Curve, Extract, Stepper, Space, ConstSpace,
    DiscreteGenerator, ConstDiscreteGenerator, Equidistant, ConstEquidistant,
    Sorted, SortedGenerator, NotSorted, TransformInput, Composite, Stack, Slice, Repeat, Wrap, Clamp};
pub use easing::{Identity};
// pub use weights::{Homogeneous, Weighted, Weights, IntoWeight};
