//! Module with structures for Homogeneous datapoints, non-uniform inerpolations, weighted interpolations
//! and adapters to handle these better.

mod homogeneous;
mod weighted;
mod weights;

pub use homogeneous::Homogeneous;
pub use weighted::Weighted;
pub use weights::{Weights, IntoWeight};
