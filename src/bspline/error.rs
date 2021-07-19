//! All error types for bspline interpolation.
#[allow(unreachable_pub)]
pub use crate::NotSorted;
#[allow(unreachable_pub)]
pub use crate::builder::{TooFewElements, TooSmallWorkspace};

use core::{fmt, convert::From};
#[cfg(feature = "std")]
use std::error::Error;

/// Errors which could occur when using or creating a linear interpolation.
#[derive(Debug, Copy, Clone)]
pub enum BSplineError {
    /// Error returned if there are too few elements to generate a curve with the necessary degree.
    TooFewElements(TooFewElements),
    /// Error returned if the workspace is not big enough.
    TooSmallWorkspace(TooSmallWorkspace),
    /// Error returned if the number of knots and elements would need a degree which is 0 or smaller.
    InvalidDegree(InvalidDegree),
    /// Error returned if knots are not sorted.
    NotSorted(NotSorted),
}

impl fmt::Display for BSplineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BSplineError::TooFewElements(inner) => inner.fmt(f),
            BSplineError::NotSorted(inner) => inner.fmt(f),
            BSplineError::InvalidDegree(inner) => inner.fmt(f),
            BSplineError::TooSmallWorkspace(inner) => inner.fmt(f),
        }
    }
}

impl From<TooFewElements> for BSplineError {
    fn from(from: TooFewElements) -> Self {
        BSplineError::TooFewElements(from)
    }
}

impl From<InvalidDegree> for BSplineError {
    fn from(from: InvalidDegree) -> Self {
        BSplineError::InvalidDegree(from)
    }
}

impl From<NotSorted> for BSplineError {
    fn from(from: NotSorted) -> Self {
        BSplineError::NotSorted(from)
    }
}

impl From<TooSmallWorkspace> for BSplineError {
    fn from(from: TooSmallWorkspace) -> Self {
        BSplineError::TooSmallWorkspace(from)
    }
}

#[cfg(feature = "std")]
impl Error for BSplineError {}

/// Error returned if the number of elements and the number of knots are not matching.
#[derive(Debug, Copy, Clone)]
pub struct InvalidDegree {
    /// The calculated degree
    degree: isize,
}

impl InvalidDegree {
    /// Create a new error with the number of elements and knots found.
    pub fn new(degree: isize) -> Self {
        InvalidDegree{
            degree,
        }
    }
}

impl fmt::Display for InvalidDegree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The degree of the resulting curve is {} and such not valid.
            Only striclty positive degrees less than the number of elements are allowed.", self.degree)
    }
}

#[cfg(feature = "std")]
impl Error for InvalidDegree {}
