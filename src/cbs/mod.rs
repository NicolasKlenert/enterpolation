//! Cubic Bezier Splines.
//!
//! The easist way to create a cubic bezier spline is by using the builder pattern of [`CBSBuilder`].
//!
//! ```rust
//! # use std::error::Error;
//! # use enterpolation::{bezier::{Bezier, BezierError}, Generator, Curve};
//! # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
//! #
//! # fn main() -> Result<(), BezierError> {
//! let spline = CBS::builder()
//!                 .elements([0.0,5.0,3.0])
//!                 .equidistant::<f64>()
//!                 .normalized()
//!                 .build()?;
//! let results = [0.0,3.25,3.0];
//! for (value,result) in spline.take(3).zip(results.iter().copied()){
//!     assert_f64_near!(value, result);
//! }
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Cubic bezier splines are curves of stitched together bezier curves of degree 3. Each bezier curve such has 4 elements.
//!
//! [`CBSBuilder`]: CBSBuilder
use crate::builder::Unknown;
use crate::{Curve, DiscreteGenerator, Generator, Space};
use core::marker::PhantomData;
use core::ops::{Mul, Sub};
use num_traits::cast::FromPrimitive;
use num_traits::real::Real;
use topology_traits::Merge;

pub struct CBS<K, E> {
    elements: E,
    knots: K,
}

impl<K, E, S, R> Generator<R> for CBS<K, E>
where
    E: DiscreteGenerator,
    E::Output: Merge<R> + Copy,
    R: Real,
    K: SortedGenerator<Output = R>,
{
    type Output = E::Output;
    fn gen(&self, scalar: R) -> E::Output {
        // The strict_upper_bound is easier to calculate and behaves nicely on the edges of the array.
        // Such it is more ergonomic than using upper_border.
        let index = self
            .knots
            .strict_upper_bound_clamped(scalar, lower_cut, upper_cut);

        // calculate cubic bezier
    }
}
