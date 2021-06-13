use thiserror::Error;
pub use crate::NotSorted;

/// Errors which could occur when using or creating a linear interpolation.
#[derive(Error, Debug)]
pub enum LinearError {
    /// Error returned if the elements are to few for a linear interpolation.
    #[error(transparent)]
    ToFewElements(#[from] ToFewElements),
    /// Error returned if a weight of zero was found.
    #[error(transparent)]
    WeightOfZero(#[from] WeightOfZero),
    /// Error returned if the number of knots and elements are not equal.
    #[error(transparent)]
    KnotElementInequality(#[from] KnotElementInequality),
    /// Error returned if knots are not sorted.
    #[error(transparent)]
    NotSorted(#[from] NotSorted),
}

/// Error returned if the elements are to few for a linear interpolation.
#[derive(Error, Debug)]
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

/// Error of weights being zero.
///
/// This is discouraged as using zero weights to ignore an element does not delete it's corresponding
/// knot, thus creating areas of undefined numbers.
#[derive(Error, Debug)]
#[error("Weights are not allowed to be zero. Instead the element should be ignored.")]
pub struct WeightOfZero {}

impl WeightOfZero {
    /// Create a new error.
    pub fn new() -> Self {
        WeightOfZero{}
    }
}

/// Error returned if the number of elements and the number of knots are not matching.
#[derive(Error, Debug)]
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
