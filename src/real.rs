//! Floating point trait
//!
//! This module will just re-export the currently used floating point trait.
//! Both for use in derive macros and for anyone who don't want to add it as an
//! additional dependency.

#[cfg(any(feature = "std", feature = "libm"))]
pub use num_traits::real::Real;

#[cfg(not(any(feature = "std", feature = "libm")))]
compile_error!(
    "The enterpolation crate needs a library for floats. Please enable the \"std\" or \"libm\" feature."
);
