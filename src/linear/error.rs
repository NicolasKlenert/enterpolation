//! All error types for linear interpolation.

pub use crate::builder::TooFewElements;
pub use crate::NotSorted;
use core::{convert::From, fmt};

#[cfg(feature = "std")]
use std::error::Error;

/// Errors which could occur when using or creating a linear interpolation.
#[derive(Debug, Copy, Clone)]
pub enum LinearError {
    /// Error returned if the elements are to few for a linear interpolation.
    ToFewElements(TooFewElements),
    /// Error returned if the number of knots and elements are not equal.
    KnotElementInequality(KnotElementInequality),
    /// Error returned if knots are not sorted.
    NotSorted(NotSorted),
}

impl fmt::Display for LinearError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinearError::ToFewElements(inner) => inner.fmt(f),
            LinearError::NotSorted(inner) => inner.fmt(f),
            LinearError::KnotElementInequality(inner) => inner.fmt(f),
        }
    }
}

impl From<TooFewElements> for LinearError {
    fn from(from: TooFewElements) -> Self {
        LinearError::ToFewElements(from)
    }
}

impl From<KnotElementInequality> for LinearError {
    fn from(from: KnotElementInequality) -> Self {
        LinearError::KnotElementInequality(from)
    }
}

impl From<NotSorted> for LinearError {
    fn from(from: NotSorted) -> Self {
        LinearError::NotSorted(from)
    }
}

#[cfg(feature = "std")]
impl Error for LinearError {}

/// Error returned if the number of elements and the number of knots are not matching.
#[derive(Debug, Copy, Clone)]
pub struct KnotElementInequality {
    /// The number of elements found.
    elements: usize,
    /// The number of knots found.
    knots: usize,
}

impl fmt::Display for KnotElementInequality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "There has to be as many knots as elements, however we found {} elements and {} knots.",
            self.elements, self.knots
        )
    }
}

#[cfg(feature = "std")]
impl Error for KnotElementInequality {}

impl KnotElementInequality {
    /// Create a new error with the number of elements and knots found.
    pub fn new(elements: usize, knots: usize) -> Self {
        KnotElementInequality { elements, knots }
    }
}
