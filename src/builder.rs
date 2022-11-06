//! Module with structures, utilities and errors used in many builders

#[cfg(any(feature = "linear", feature = "bezier", feature = "bspline"))]
use core::fmt;
#[cfg(any(feature = "linear", feature = "bezier", feature = "bspline"))]
use core::marker::PhantomData;

#[cfg(all(
    feature = "std",
    any(feature = "linear", feature = "bezier", feature = "bspline")
))]
use std::error::Error;

/// Struct indicator to mark that we don't use weights.
#[cfg(any(feature = "linear", feature = "bezier", feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WithoutWeight;

/// Struct indicator to mark that we use weights.
#[cfg(any(feature = "linear", feature = "bezier", feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct WithWeight;

/// Struct indicator to mark information not yet given.
#[cfg(any(feature = "linear", feature = "bezier", feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Unknown;

/// Struct to indicate to use normalized Input
#[cfg(feature = "bezier")]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NormalizedInput<R = f64>(PhantomData<*const R>);

#[cfg(feature = "bezier")]
impl<R> NormalizedInput<R> {
    pub const fn new() -> Self {
        NormalizedInput(PhantomData)
    }
}

/// Struct to indicate which input domain to use
#[cfg(feature = "bezier")]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct InputDomain<R = f64> {
    pub start: R,
    pub end: R,
}

#[cfg(feature = "bezier")]
impl<R> InputDomain<R> {
    pub fn new(start: R, end: R) -> Self {
        InputDomain { start, end }
    }
}

/// Struct indicator to mark which type to use
#[cfg(any(feature = "linear", feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Type<R = f64>(PhantomData<*const R>);

#[cfg(any(feature = "linear", feature = "bspline"))]
impl<R> Type<R> {
    pub const fn new() -> Self {
        Type(PhantomData)
    }
}

/// Error returned if if there are no elements.
#[cfg(feature = "bezier")]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Empty {}

#[cfg(feature = "bezier")]
impl Empty {
    /// Create a new error.
    pub const fn new() -> Self {
        Empty {}
    }
}

#[cfg(feature = "bezier")]
impl fmt::Display for Empty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No elements given, an empty generator is not allowed.")
    }
}

#[cfg(all(feature = "std", feature = "bezier"))]
impl Error for Empty {}

/// Error returned if the elements are to few for the specific interpolation.
#[cfg(any(feature = "linear", feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TooFewElements {
    /// The number of elements found.
    found: usize,
}

#[cfg(any(feature = "linear", feature = "bspline"))]
impl fmt::Display for TooFewElements {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "To few elements given for the interpolation. {} elements were given, but at least 2 are necessary.", self.found)
    }
}

#[cfg(all(feature = "std", any(feature = "linear", feature = "bspline")))]
impl Error for TooFewElements {}

#[cfg(any(feature = "linear", feature = "bspline"))]
impl TooFewElements {
    /// Create a new error and document the number of elements found.
    pub fn new(found: usize) -> Self {
        TooFewElements { found }
    }
}

/// Error returned when the number of knots are too few.
#[cfg(feature = "bspline")]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TooFewKnots {
    /// The number of knots found.
    found: usize,
}

#[cfg(feature = "bspline")]
impl fmt::Display for TooFewKnots {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "To few knots given for the interpolation. {} knots were given, but at least 2 are necessary.", self.found)
    }
}

#[cfg(all(feature = "std", feature = "bspline"))]
impl Error for TooFewKnots {}

#[cfg(feature = "bspline")]
impl TooFewKnots {
    /// Create a new error and document the number of knots found.
    pub fn new(found: usize) -> Self {
        TooFewKnots { found }
    }
}

/// Error returned when the workspace is too small.
#[cfg(any(feature = "bezier", feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct TooSmallWorkspace {
    found: usize,
    necessary: usize,
}

#[cfg(any(feature = "bezier", feature = "bspline"))]
impl fmt::Display for TooSmallWorkspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "The given workspace is too small with space for {} elements, at least {} have to fit.",
            self.found, self.necessary
        )
    }
}

#[cfg(all(feature = "std", any(feature = "bezier", feature = "bspline")))]
impl Error for TooSmallWorkspace {}

#[cfg(any(feature = "bezier", feature = "bspline"))]
impl TooSmallWorkspace {
    /// Create a new error.
    pub fn new(found: usize, necessary: usize) -> Self {
        TooSmallWorkspace { found, necessary }
    }
}
