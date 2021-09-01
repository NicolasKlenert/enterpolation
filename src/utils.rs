//! Module for different utilities which are used across other modules or to help the user of the library.
use num_traits::real::Real;
use core::ops::{Add,Mul};

/// Linear interpolation of the two values given.
pub fn lerp<T,R>(first: T, second: T, factor: R) -> T
where
    T: Add<Output = T> + Mul<R,Output = T>,
    R: Real
{
    first * (R::one()-factor) + second * factor
}
