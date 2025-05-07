//! Module with structures for homogeneous datapoints, non-uniform inerpolations, weighted interpolations
//! and adapters to handle these better.

mod homogeneous;
mod weighted;

pub use homogeneous::Homogeneous;
pub use weighted::Weighted;

use crate::{Chain, ConstChain, Curve, Signal};
use core::ops::Mul;
use num_traits::identities::Zero;
use num_traits::real::Real;

/// Signal adaptor to transform `(T,R)` to `Homogeneous<T,R>`.
///
/// Weights given by the signal who equal `R::zero()` are considered to be at infinity.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Weights<G> {
    signal: G,
}

impl<G> Weights<G> {
    /// Transform given signal such that it outputs homogeneous data.
    pub fn new(signal: G) -> Self {
        Weights { signal }
    }
}

impl<G, Input> Signal<Input> for Weights<G>
where
    G: Signal<Input>,
    G::Output: IntoWeight,
{
    type Output =
        Homogeneous<<G::Output as IntoWeight>::Element, <G::Output as IntoWeight>::Weight>;
    fn eval(&self, input: Input) -> Self::Output {
        self.signal.eval(input).into_weight()
    }
}

impl<G> Chain for Weights<G>
where
    G: Chain,
    G::Output: IntoWeight,
{
    fn len(&self) -> usize {
        self.signal.len()
    }
}

impl<G, const N: usize> ConstChain<N> for Weights<G>
where
    G: ConstChain<N>,
    G::Output: IntoWeight,
{
}

impl<C, R> Curve<R> for Weights<C>
where
    C: Curve<R>,
    C::Output: IntoWeight,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        self.signal.domain()
    }
}

/// Trait for all structs which can be transformed into homogeneous data.
///
/// This trait is used to be able to implement Signals for Weights without having to add other generic variables.
pub trait IntoWeight {
    /// The element/direction of the homogeneous data.
    type Element;
    /// The weight/rational of the homogeneous data.
    type Weight;
    /// Method to convert self into homogeneous data.
    fn into_weight(self) -> Homogeneous<Self::Element, Self::Weight>;
}

impl<T, R> IntoWeight for (T, R)
where
    T: Mul<R, Output = T>,
    R: Zero + Copy,
{
    type Element = T;
    type Weight = R;
    fn into_weight(self) -> Homogeneous<T, R> {
        Homogeneous::weighted_or_infinite(self.0, self.1)
    }
}

impl<T, R> IntoWeight for Homogeneous<T, R>
where
    T: Mul<R, Output = T>,
    R: Zero + Copy,
{
    type Element = T;
    type Weight = R;
    fn into_weight(self) -> Homogeneous<T, R> {
        self
    }
}
