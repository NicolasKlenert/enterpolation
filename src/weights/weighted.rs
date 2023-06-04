//! The adaptor `Weighted` can be used for all interpolations to hide the inner workings of a weighted element.

use crate::weights::Homogeneous;
use crate::{Curve, Generator};
use core::ops::Div;
use num_traits::real::Real;

/// Interpolation Adaptor used for weighted elements to automatically unwrap them from their weights.
///
/// This Adaptor is often appended to an interpolation with weighted elements to automatically unwrap them.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Weighted<G> {
    inner: G,
}

impl<G> Weighted<G> {
    /// Use the `Weighted` Adaptor on the given weighted interpolation to automatically unwrap the elements of their weight.
    pub fn new(gen: G) -> Self {
        Weighted { inner: gen }
    }
    /// Return the inner interpolation.
    pub fn inner(self) -> G {
        self.inner
    }
}

impl<G, I> Generator<I> for Weighted<G>
where
    G: Generator<I>,
    G::Output: Project,
{
    type Output = <G::Output as Project>::Element;
    fn gen(&self, input: I) -> Self::Output {
        self.inner.gen(input).project()
    }
}

impl<G, R> Curve<R> for Weighted<G>
where
    G: Curve<R>,
    G::Output: Project,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        self.inner.domain()
    }
}

/// This trait is used to be able to implement Generator for Weights without having to add other generic variables.
pub trait Project {
    type Element;
    type Weight;
    fn project(self) -> Self::Element;
}

impl<T, R> Project for Homogeneous<T, R>
where
    T: Div<R, Output = T>,
{
    type Element = T;
    type Weight = R;
    fn project(self) -> Self::Element {
        self.project()
    }
}
