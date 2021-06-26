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
    NonStrictPositiveDegree(#[from] NonStrictPositiveDegree),
    /// Error returned if knots are not sorted.
    #[error(transparent)]
    NotSorted(#[from] NotSorted),
}

/// Error returned if the number of elements and the number of knots are not matching.
#[derive(Error, Debug, Copy, Clone)]
#[error("Degree is calculated witht the number of elements and knots,
    however we found {elements} elements and {knots} knots, such resulting in a degree of {degree}
    which is forbidden.")]
pub struct NonStrictPositiveDegree {
    /// The number of elements found.
    elements: usize,
    /// The number of knots found.
    knots: usize,
    /// The calculated degree
    degree: isize,
}

impl NonStrictPositiveDegree {
    /// Create a new error with the number of elements and knots found.
    pub fn new(elements: usize, knots: usize) -> Self {
        NonStrictPositiveDegree{
            elements,
            knots,
            degree: knots as isize - elements as isize -1,
        }
    }
}
