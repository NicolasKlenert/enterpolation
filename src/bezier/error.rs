#[allow(unreachable_pub)]
pub use crate::builder::{Empty, TooSmallWorkspace};

use core::{convert::From, fmt};

#[cfg(feature = "std")]
use std::error::Error;

/// Errors which could occur when using or creating a linear interpolation.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum BezierError {
    /// Error returned if the generator does not contain any elements.
    Empty(Empty),
    /// Error returned if the given workspace is too small for the interpolation to use.
    TooSmallWorkspace(TooSmallWorkspace),
}

impl fmt::Display for BezierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BezierError::Empty(inner) => inner.fmt(f),
            BezierError::TooSmallWorkspace(inner) => inner.fmt(f),
        }
    }
}

impl From<Empty> for BezierError {
    fn from(from: Empty) -> Self {
        BezierError::Empty(from)
    }
}

impl From<TooSmallWorkspace> for BezierError {
    fn from(from: TooSmallWorkspace) -> Self {
        BezierError::TooSmallWorkspace(from)
    }
}

#[cfg(feature = "std")]
impl Error for BezierError {}
