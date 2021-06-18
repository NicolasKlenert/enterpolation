//! All error types for linear interpolation.

use thiserror::Error;
pub use crate::NotSorted;

/// Errors which could occur when using or creating a linear interpolation.
#[derive(Error, Debug, Copy, Clone)]
pub enum LinearError {
    /// Error returned if the elements are to few for a linear interpolation.
    #[error(transparent)]
    ToFewElements(#[from] ToFewElements),
    /// Error returned if the number of knots and elements are not equal.
    #[error(transparent)]
    KnotElementInequality(#[from] KnotElementInequality),
    /// Error returned if knots are not sorted.
    #[error(transparent)]
    NotSorted(#[from] NotSorted),
}

/// Error returned if the elements are to few for a linear interpolation.
#[derive(Error, Debug, Copy, Clone)]
#[error("To few elements given for a linear interpolation. {found} elements were given, but at least 2 are necessary.")]
pub struct ToFewElements {
    /// The number of elements found.
    found: usize,
}

impl ToFewElements {
    /// Create a new error and document the number of elements found.
    pub fn new(found: usize) -> Self {
        ToFewElements{
            found
        }
    }
}

/// Error returned if the number of elements and the number of knots are not matching.
#[derive(Error, Debug, Copy, Clone)]
#[error("There has to be as many knots as elements, however we found {elements} elements and {knots} knots.")]
pub struct KnotElementInequality {
    /// The number of elements found.
    elements: usize,
    /// The number of knots found.
    knots: usize,
}

impl KnotElementInequality {
    /// Create a new error with the number of elements and knots found.
    pub fn new(elements: usize, knots: usize) -> Self {
        KnotElementInequality{
            elements,
            knots,
        }
    }
}
