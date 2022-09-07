//! All error types for bspline interpolation.
#[allow(unreachable_pub)]
pub use crate::builder::{TooFewElements, TooFewKnots, TooSmallWorkspace};
#[allow(unreachable_pub)]
pub use crate::NotSorted;

use core::{convert::From, fmt};
#[cfg(feature = "std")]
use std::error::Error;

// use super::BSpline;

/// Errors which could occur when using or creating a linear interpolation.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BSplineError {
    /// Error returned if there are too few elements.
    TooFewElements(TooFewElements),
    /// Error returned if there are too few knots.
    TooFewKnots(TooFewKnots),
    /// Error returned if the workspace is not big enough.
    TooSmallWorkspace(TooSmallWorkspace),
    /// Error returned when the degree is 0 or smaller.
    InvalidDegree(InvalidDegree),
    /// Error returned if knots are not sorted.
    NotSorted(NotSorted),
    /// Error returned when the number of elements, knots and degree are incongruous.
    IncongruousParams(IncongruousParams),
}

impl fmt::Display for BSplineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BSplineError::TooFewElements(inner) => inner.fmt(f),
            BSplineError::NotSorted(inner) => inner.fmt(f),
            BSplineError::InvalidDegree(inner) => inner.fmt(f),
            BSplineError::TooSmallWorkspace(inner) => inner.fmt(f),
            BSplineError::TooFewKnots(inner) => inner.fmt(f),
            BSplineError::IncongruousParams(inner) => inner.fmt(f),
        }
    }
}

impl From<TooFewElements> for BSplineError {
    fn from(from: TooFewElements) -> Self {
        BSplineError::TooFewElements(from)
    }
}

impl From<TooFewKnots> for BSplineError {
    fn from(from: TooFewKnots) -> Self {
        BSplineError::TooFewKnots(from)
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

impl From<IncongruousParams> for BSplineError {
    fn from(from: IncongruousParams) -> Self {
        BSplineError::IncongruousParams(from)
    }
}

#[cfg(feature = "std")]
impl Error for BSplineError {}

/// Error returned if the number of elements and the number of knots are not matching.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct InvalidDegree {
    /// The calculated degree
    degree: usize,
}

impl InvalidDegree {
    /// Create a new error with the number of elements and knots found.
    pub fn new(degree: usize) -> Self {
        InvalidDegree { degree }
    }
}

impl fmt::Display for InvalidDegree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The degree of the resulting curve is {} and such not valid.
            Only striclty positive degrees less than the number of elements are allowed.",
            self.degree
        )
    }
}

#[cfg(feature = "std")]
impl Error for InvalidDegree {}

#[derive(Debug, Copy, Clone)]
enum BSplineBuildMode {
    Open,
    Clamped,
    Legacy,
}

/// Error returned when the number of elements, knots and the degree are not matching.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IncongruousParams {
    elements: usize,
    knots: isize,
    degree: isize,
    mode: BSplineBuildMode,
}

impl IncongruousParams {
    /// Invalid values for an open bspline
    pub fn open(elements: usize, knots: isize, degree: isize) -> Self {
        IncongruousParams {
            elements,
            knots,
            degree,
            mode: BSplineBuildMode::Open,
        }
    }
    /// Invalid values for a clamped bspline
    pub fn clamped(elements: usize, knots: isize, degree: isize) -> Self {
        IncongruousParams {
            elements,
            knots,
            degree,
            mode: BSplineBuildMode::Clamped,
        }
    }
    /// Invalid values for a legacy bspline
    pub fn legacy(elements: usize, knots: isize, degree: isize) -> Self {
        IncongruousParams {
            elements,
            knots,
            degree,
            mode: BSplineBuildMode::Legacy,
        }
    }
}

impl fmt::Display for IncongruousParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mode {
            BSplineBuildMode::Open => {
                write!(
                    f,
                    "Found {} elements, {} knots and a degree of {}. 
                    However, the formula for an open bspline is 
                    #knots = #elements + degree - 1 or
                    degree = #knots - #elements + 1.",
                    self.elements, self.knots, self.degree
                )
            }
            BSplineBuildMode::Clamped => {
                write!(
                    f,
                    "Found {} elements, {} knots and a degree of {}. 
                    However, the formula for a clamped bspline is 
                    #knots = #elements - degree + 1 or
                    degree = #elements - #knots + 1.",
                    self.elements, self.knots, self.degree
                )
            }
            BSplineBuildMode::Legacy => {
                write!(
                    f,
                    "Found {} elements, {} knots and a degree of {}. 
                    However, the formula for a legacy bspline is 
                    #knots = #elements + degree + 1 or
                    degree = #knots - #elements - 1.",
                    self.elements, self.knots, self.degree
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl Error for IncongruousParams {}
