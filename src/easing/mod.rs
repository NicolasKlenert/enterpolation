//! Module for easing functions.
//!
//! Easing function, in the context of this crate, are function which take as only input
//! a real number in [0.0,1.0] and return a real number in [0.0,1.0].

use crate::{Curve, Signal};
use num_traits::FromPrimitive;
use num_traits::real::Real;

mod plateau;
pub use plateau::Plateau;

/// This is just a wrapper for easing functions.
///
/// We expect the domain to be normalized.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FuncEase<F> {
    func: F,
}

impl<F> FuncEase<F> {
    /// Create a new struct from the given function.
    pub fn new(func: F) -> Self {
        FuncEase { func }
    }
}

impl<F, R> Signal<R> for FuncEase<F>
where
    F: Fn(R) -> R,
{
    type Output = R;
    fn eval(&self, input: R) -> R {
        (self.func)(input)
    }
}

impl<F, R> Curve<R> for FuncEase<F>
where
    F: Fn(R) -> R,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        [R::zero(), R::one()]
    }
}

/// Identity as Curve.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Identity {}

impl Identity {
    /// Create a new Identity struct.
    pub const fn new() -> Self {
        Identity {}
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> Signal<R> for Identity {
    type Output = R;
    fn eval(&self, input: R) -> R {
        input
    }
}

impl<R> Curve<R> for Identity
where
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        [R::zero(), R::one()]
    }
}

/// Flips the "start" and "end".
///
/// For easing functions seen as a graph, this flips the graph on the x axis.
pub fn flip<R>(x: R) -> R
where
    R: Real,
{
    R::one() - x
}

/// Smoothstart, also known as ease-in, smooths out the start of the graph.
pub fn smoothstart<R, const N: usize>(x: R) -> R
where
    R: Real,
{
    let mut mul = x;
    for _ in 1..N {
        mul = mul * x;
    }
    mul
}

/// Smoothend, also known as ease-out, smooths out the end of the graph.
pub fn smoothend<R, const N: usize>(x: R) -> R
where
    R: Real,
{
    flip(smoothstart::<R, N>(flip(x)))
}

/// Smoothstep function, see <https://en.wikipedia.org/wiki/Smoothstep>
pub fn smoothstep<R>(x: R) -> R
where
    R: Real + FromPrimitive,
{
    let two = R::from_usize(2).expect("Could not convert 2 to a real number");
    let three = R::from_usize(3).expect("Could not convert 3 to a real number");
    x * x * (three - two * x)
}

/// A smoother variant of the smoothstep function, see <https://en.wikipedia.org/wiki/Smoothstep>
pub fn smootherstep<R>(x: R) -> R
where
    R: Real + FromPrimitive,
{
    let six = R::from_usize(6).expect("Could not convert 6 to a real number");
    let ten = R::from_usize(10).expect("Could not convert 10 to a real number");
    let fifteen = R::from_usize(15).expect("Could not convert 15 to a real number");
    x * x * x * (x * (x * six - fifteen) + ten)
}
