use thiserror::Error;
#[allow(unreachable_pub)]
pub use crate::builder::{Empty, TooSmallWorkspace};

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
