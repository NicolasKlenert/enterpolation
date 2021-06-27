//! Module with structures, utilities and errors used in many builders

use core::marker::PhantomData;
use thiserror::Error;

// pub trait WeightMarker{}

/// Struct indicator to mark that we don't use weights
#[derive(Debug, Copy, Clone)]
pub struct WithoutWeight;

/// Struct indicator to mark that we use weights
#[derive(Debug, Copy, Clone)]
pub struct WithWeight;

/// Struct indicator to mark information not yet given.
#[derive(Debug, Copy, Clone)]
pub struct Unknown;

/// Struct to indicate to use normalized Input
#[derive(Debug, Copy, Clone)]
pub struct NormalizedInput<R = f64>(PhantomData<*const R>);

impl<R> NormalizedInput<R> {
    pub const fn new() -> Self {
        NormalizedInput(PhantomData)
    }
}

/// Struct to indicate which input domain to use
#[derive(Debug, Copy, Clone)]
pub struct InputDomain<R = f64>{
    pub start: R,
    pub end: R,
}

impl<R> InputDomain<R>{
    pub fn new(start: R, end: R) -> Self {
        InputDomain{
            start,
            end
        }
    }
}

/// Struct indicator to mark which type to use
#[derive(Debug, Copy, Clone)]
pub struct Type<R = f64>(PhantomData<*const R>);

impl<R> Type<R> {
    pub const fn new() -> Self {
        Type(PhantomData)
    }
}

/// Error returned if if there are no elements.
#[derive(Error, Debug, Copy, Clone)]
#[error("No elements given, an empty generator is not allowed.")]
pub struct Empty {}

impl Empty {
    /// Create a new error.
    pub const fn new() -> Self {
        Empty{}
    }
}

/// Error returned if if there are no elements.
#[derive(Error, Debug, Copy, Clone)]
#[error("The given workspace is too small with space for {found} elements, at least {necessary} have to fit.")]
pub struct TooSmallWorkspace {
    found: usize,
    necessary: usize,
}

impl TooSmallWorkspace {
    /// Create a new error.
    pub fn new(found: usize, necessary: usize) -> Self {
        TooSmallWorkspace{
            found,
            necessary
        }
    }
}

//TOOD: add error WrongCallingOrder and implement it for all methods of the builder where a function is called too early!
