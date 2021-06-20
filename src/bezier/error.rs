use thiserror::Error;

/// Errors which could occur when using or creating a linear interpolation.
#[derive(Error, Debug, Copy, Clone)]
pub enum BezierError {
    /// Error returned if the generator does not contain any elements.
    #[error(transparent)]
    Empty(#[from] Empty),
    /// Error returned if the given workspace is too small for the interpolation to use.
    #[error(transparent)]
    TooSmallWorkspace(#[from] TooSmallWorkspace),
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
