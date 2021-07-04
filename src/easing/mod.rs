//! Module for easing functions.
//!
//! Easing function, in the context of this crate, are function which take as only input
//! a real number in [0.0,1.0] and return a real number in [0.0,1.0].

use num_traits::real::Real;
use num_traits::FromPrimitive;
use crate::{Generator, Interpolation, Curve};

mod plateau;
pub use plateau::Plateau;

/// Marker trait for special curves.
///
/// Curves are Easing if and only if their domain is [0.0,1.0] and their output is also in [0.0,1.0]
/// as well as their output for 0.0 has to be 0.0 and their output for 1.0 has to be 1.0.
pub trait Easing<R> : Curve<R>
where R: Real
{}

/// This is just a wrapper for easing functions.
///
/// This wrapper should only ever constructed with easing functions, as this would otherwise
/// wrongly implement the Easing trait.
#[derive(Debug, Clone, Copy)]
pub struct FuncEase<F> {
    func: F,
}

impl<F> FuncEase<F>{
    /// Create a new struct which implements the `Easing` trait.
    ///
    /// The given function should hold the requirements of an easing function.
    pub fn new(func: F) -> Self {
        FuncEase {
            func
        }
    }
}

impl<F,R> Generator<R> for FuncEase<F>
where F: Fn(R) -> R,
{
    type Output = R;
    fn gen(&self, input: R) -> R{
        (self.func)(input)
    }
}

impl<F,R> Interpolation<R> for FuncEase<F>
where F: Fn(R) -> R {}

impl<F,R> Curve<R> for FuncEase<F>
where
    F: Fn(R) -> R,
    R: Real,
{
    fn domain(&self) -> [R;2] {
        [R::zero(),R::one()]
    }
}

impl<F,R> Easing<R> for FuncEase<F>
where
    F: Fn(R) -> R,
    R: Real,
{}

pub struct Identity{}

impl Identity {
    pub const fn new() -> Self {
        Identity {}
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> Generator<R> for Identity
{
    type Output = R;
    fn gen(&self, input: R) -> R{
        input
    }
}

impl<R> Interpolation<R> for Identity {}

impl<R> Curve<R> for Identity
where
    R: Real,
{
    fn domain(&self) -> [R;2] {
        [R::zero(),R::one()]
    }
}

impl<R> Easing<R> for Identity where R: Real {}

/// Flips the "start" and "end".
///
/// For easing functions seen as a graph, this flips the graph on the x axis.
pub fn flip<R>(x: R) -> R
where R: Real
{
    R::one() - x
}

/// Smoothstart, also known as ease-in, smooths out the start of the graph.
pub fn smoothstart<R,const N: usize>(x: R) -> R
where R: Real
{
    let mut mul = x;
    for _ in 1..N{
        mul = mul * x;
    }
    mul
}

/// Smoothend, also known as ease-out, smooths out the end of the graph.
pub fn smoothend<R,const N: usize>(x: R) -> R
where R: Real
{
    flip(smoothstart::<R,N>(flip(x)))
}

/// Smoothstep function, see https://en.wikipedia.org/wiki/Smoothstep
pub fn smoothstep<R>(x: R) -> R
where R: Real + FromPrimitive,
{
    let two = R::from_usize(2).expect("Could not convert 2 to a real number");
    let three = R::from_usize(3).expect("Could not convert 3 to a real number");
    x * x * (three - two * x)
}

/// A smoother variant of the smoothstep function, see https://en.wikipedia.org/wiki/Smoothstep
pub fn smootherstep<R>(x: R) -> R
where R: Real + FromPrimitive,
{
    let six = R::from_usize(6).expect("Could not convert 6 to a real number");
    let ten = R::from_usize(10).expect("Could not convert 10 to a real number");
    let fifteen = R::from_usize(15).expect("Could not convert 15 to a real number");
    x * x * x *( x * ( x * six - fifteen ) + ten)
}
