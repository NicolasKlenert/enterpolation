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
    /// Error returned when elements and knots are not matching together.
    IncongruousElementsKnots(IncongruousElementsKnots),
    /// Error returned when elements and degree are ill-matched.
    IncongruousElementsDegree(IncongruousElementsDegree),
}

impl fmt::Display for BSplineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BSplineError::TooFewElements(inner) => inner.fmt(f),
            BSplineError::NotSorted(inner) => inner.fmt(f),
            BSplineError::InvalidDegree(inner) => inner.fmt(f),
            BSplineError::TooSmallWorkspace(inner) => inner.fmt(f),
            BSplineError::TooFewKnots(inner) => inner.fmt(f),
            BSplineError::IncongruousElementsKnots(inner) => inner.fmt(f),
            BSplineError::IncongruousElementsDegree(inner) => inner.fmt(f),
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

impl From<IncongruousElementsKnots> for BSplineError {
    fn from(from: IncongruousElementsKnots) -> Self {
        BSplineError::IncongruousElementsKnots(from)
    }
}

impl From<IncongruousElementsDegree> for BSplineError {
    fn from(from: IncongruousElementsDegree) -> Self {
        BSplineError::IncongruousElementsDegree(from)
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum BSplineBuildMode {
    Open,
    Clamped,
    Legacy,
}

/// Error returned when the number of elements and knots are ill-matched.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IncongruousElementsKnots {
    elements: usize,
    knots: usize,
    mode: BSplineBuildMode,
}

impl IncongruousElementsKnots {
    /// Invalid values for an open bspline
    pub fn open(elements: usize, knots: usize) -> Self {
        IncongruousElementsKnots {
            elements,
            knots,
            mode: BSplineBuildMode::Open,
        }
    }
    /// Invalid values for a clamped bspline
    pub fn clamped(elements: usize, knots: usize) -> Self {
        IncongruousElementsKnots {
            elements,
            knots,
            mode: BSplineBuildMode::Clamped,
        }
    }
    /// Invalid values for a legacy bspline
    pub fn legacy(elements: usize, knots: usize) -> Self {
        IncongruousElementsKnots {
            elements,
            knots,
            mode: BSplineBuildMode::Legacy,
        }
    }
}

impl fmt::Display for IncongruousElementsKnots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mode {
            BSplineBuildMode::Open => {
                write!(
                    f,
                    "Found {} elements (#e) and {} knots (#k), but for an open bspline 
                    #e <= #k <= 2*(#e-1) must hold.",
                    self.elements, self.knots
                )
            }
            BSplineBuildMode::Clamped => {
                write!(
                    f,
                    "Found {} elements and {} knots, but for a clamped bspline there 
                    must be at least as many elements as there are knots.",
                    self.elements, self.knots
                )
            }
            BSplineBuildMode::Legacy => {
                write!(
                    f,
                    "Found {} elements (#e) and {} knots (#k), but for a legacy bspline 
                    #e+2 <= #k <= 2*(#e+1) must hold.",
                    self.elements, self.knots
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl Error for IncongruousElementsKnots {}

/// Error returned when the number of elements and the degree are ill-matched.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IncongruousElementsDegree {
    elements: usize,
    degree: usize,
    mode: BSplineBuildMode,
}

impl IncongruousElementsDegree {
    /// Invalid values for an open bspline
    pub fn open(elements: usize, degree: usize) -> Self {
        IncongruousElementsDegree {
            elements,
            degree,
            mode: BSplineBuildMode::Open,
        }
    }
    /// Invalid values for a clamped bspline
    pub fn clamped(elements: usize, degree: usize) -> Self {
        IncongruousElementsDegree {
            elements,
            degree,
            mode: BSplineBuildMode::Clamped,
        }
    }
    /// Invalid values for a legacy bspline
    pub fn legacy(elements: usize, degree: usize) -> Self {
        IncongruousElementsDegree {
            elements,
            degree,
            mode: BSplineBuildMode::Legacy,
        }
    }
}

impl fmt::Display for IncongruousElementsDegree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mode {
            BSplineBuildMode::Open => {
                write!(
                    f,
                    "Found {} elements and degree of {}, but for an open bspline 
                    there must be more elements than the degree of the spline.",
                    self.elements, self.degree
                )
            }
            BSplineBuildMode::Clamped => {
                write!(
                    f,
                    "Found {} elements and a degree of {}. 
                    However, the degree of a clamped bspline 
                    must be less than the number of elements.",
                    self.elements, self.degree
                )
            }
            BSplineBuildMode::Legacy => {
                write!(
                    f,
                    "Found {} elements and degree  of {}, but for a legacy bspline 
                    there must be more elements than the degree of the spline.",
                    self.elements, self.degree
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl Error for IncongruousElementsDegree {}
