//! All error types for linear interpolation.

use thiserror::Error;
#[allow(unreachable_pub)]
pub use crate::NotSorted;
#[allow(unreachable_pub)]
pub use crate::builder::{Empty, TooSmallWorkspace};

/// Errors which could occur when using or creating a linear interpolation.
#[derive(Error, Debug, Copy, Clone)]
pub enum BSplineError {
    /// Error returned if no elements are in the generator.
    #[error(transparent)]
    Empty(#[from] Empty),
    /// Error returned if the workspace is not big enough.
    #[error(transparent)]
    TooSmallWorkspace(#[from] TooSmallWorkspace),
    /// Error returned if the number of knots and elements would need a degree which is 0 or smaller.
    #[error(transparent)]
    NonValidDegree(#[from] NonValidDegree),
    /// Error returned if knots are not sorted.
    #[error(transparent)]
    NotSorted(#[from] NotSorted),
}

/// Error returned if the number of elements and the number of knots are not matching.
#[derive(Error, Debug, Copy, Clone)]
#[error("The degree of the resulting curve is {degree} and such not valid.
    Only striclty positive degrees less than the number of elements are allowed.")]
pub struct NonValidDegree {
    /// The calculated degree
    degree: isize,
}

impl NonValidDegree {
    /// Create a new error with the number of elements and knots found.
    pub fn new(degree: isize) -> Self {
        NonValidDegree{
            degree,
        }
    }
}
