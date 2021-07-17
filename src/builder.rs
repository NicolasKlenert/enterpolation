//! Module with structures, utilities and errors used in many builders

use core::marker::PhantomData;
use core::fmt;
#[cfg(any(feature = "linear", feature = "bspline"))]
use crate::{Generator, DiscreteGenerator};

#[cfg(feature = "std")]
use std::error::Error;

/// Struct indicator to mark that we don't use weights.
#[derive(Debug, Copy, Clone)]
pub struct WithoutWeight;

/// Struct indicator to mark that we use weights.
#[derive(Debug, Copy, Clone)]
pub struct WithWeight;

/// Struct indicator that knots are not guaranteed to be sorted.
#[cfg(any(feature = "linear", feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
pub struct Unsorted<K>(pub K);

#[cfg(any(feature = "linear", feature = "bspline"))]
impl<K> Unsorted<K> {
    pub const fn new(knots: K) -> Self {
        Unsorted(knots)
    }
}

#[cfg(any(feature = "linear", feature = "bspline"))]
impl<K> Generator<usize> for Unsorted<K>
where K: Generator<usize>
{
    type Output = K::Output;
    fn gen(&self, value: usize) -> Self::Output {
        self.0.gen(value)
    }
}

#[cfg(any(feature = "linear", feature = "bspline"))]
impl<K> DiscreteGenerator for Unsorted<K>
where K: DiscreteGenerator
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Struct indicator to mark information not yet given.
#[derive(Debug, Copy, Clone)]
pub struct Unknown;

/// Struct to indicate to use normalized Input
#[cfg(feature = "bezier")]
#[derive(Debug, Copy, Clone)]
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
pub struct InputDomain<R = f64>{
    pub start: R,
    pub end: R,
}

#[cfg(feature = "bezier")]
impl<R> InputDomain<R>{
    pub fn new(start: R, end: R) -> Self {
        InputDomain{
            start,
            end
        }
    }
}

/// Struct indicator to mark which type to use
#[cfg(any(feature = "linear",feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
pub struct Type<R = f64>(PhantomData<*const R>);

#[cfg(any(feature = "linear",feature = "bspline"))]
impl<R> Type<R> {
    pub const fn new() -> Self {
        Type(PhantomData)
    }
}

/// Error returned if if there are no elements.
#[cfg(feature = "bezier")]
#[derive(Debug, Copy, Clone)]
pub struct Empty {}

#[cfg(feature = "bezier")]
impl Empty {
    /// Create a new error.
    pub const fn new() -> Self {
        Empty{}
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
pub struct TooFewElements {
    /// The number of elements found.
    found: usize,
}

#[cfg(any(feature = "linear", feature = "bspline"))]
impl fmt::Display for TooFewElements {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "To few elements given for a linear interpolation. {} elements were given, but at least 2 are necessary.", self.found)
    }
}

#[cfg(all(feature = "std",any(feature = "linear", feature = "bspline")))]
impl Error for TooFewElements {}

#[cfg(any(feature = "linear", feature = "bspline"))]
impl TooFewElements {
    /// Create a new error and document the number of elements found.
    pub fn new(found: usize) -> Self {
        TooFewElements{
            found
        }
    }
}

/// Error returned if if there are no elements.
#[cfg(any(feature = "bezier", feature = "bspline"))]
#[derive(Debug, Copy, Clone)]
pub struct TooSmallWorkspace {
    found: usize,
    necessary: usize,
}

#[cfg(any(feature = "bezier", feature = "bspline"))]
impl fmt::Display for TooSmallWorkspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The given workspace is too small with space for {} elements, at least {} have to fit.", self.found, self.necessary)
    }
}

#[cfg(all(feature = "std",any(feature = "bezier", feature = "bspline")))]
impl Error for TooSmallWorkspace {}

#[cfg(any(feature = "bezier", feature = "bspline"))]
impl TooSmallWorkspace {
    /// Create a new error.
    pub fn new(found: usize, necessary: usize) -> Self {
        TooSmallWorkspace{
            found,
            necessary
        }
    }
}
