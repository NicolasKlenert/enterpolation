//! This crate provides a myriad of different interpolation, extrapolation and animation methods.
//! Most notably it tries to be generic and modular. If instances of your type act somewhat like
//! a vector space, this crate will be able to interpolate, extrapolate and animate them.
//! TODO: describe more

//TODO: Also our min and max_index of our linear interpolation does NOT clamp the values together...array out of bounds!
//TODO: SO upper_border can be broken somehow! Look into it!

//TODO: all interpolations should have as knots field not just K but Sorted(NonEmpty(K))
//TODO: all interpolations should have as elements field not just E but NonEmpty(E)
//TODO: we want to achieve many different creation options such that a builder will be necessary
//TODO: for now, create a builder for each different interpolation!
//TODO: Afterwards delete the implementation of SortedList for array and vec
//TODO: and add NonEmpty as super trait for SortedList!

#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    nonstandard_style,
    rust_2018_idioms,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unused_extern_crates,
    unused_qualifications,
    variant_size_differences
)]

#[macro_use]
extern crate assert_float_eq;

pub mod linear;
pub mod bezier;
pub mod bspline;
pub mod utils;
pub mod weights;

mod real;
mod never;
mod base;
mod builder;

use crate::real::Real;
pub use base::{Generator, Interpolation, Curve, Extract, Stepper, Space, ConstSpace, DynSpace,
    DiscreteGenerator, ConstDiscreteGenerator, Equidistant, ConstEquidistant,
    Sorted, SortedGenerator, NotSorted};
pub use weights::{Homogeneous, Weighted, Weights, IntoWeight};
use core::ops::{Add, Mul};

/// Struct which transforms the input before sending it to the underlying generator.
///
/// Both addition and multiplication is done. In regards to math operation priorities, multiplication is done first.
#[derive(Clone, Debug)]
pub struct TransformInput<G,A,M>{
    addition: A,
    multiplication: M,
    inner: G
}

impl<G,A,M> TransformInput<G,A,M>{
    /// Create a generic `TransformInput`.
    pub fn new(generator: G, addition: A, multiplication: M) -> Self {
        TransformInput {
            inner: generator,
            addition,
            multiplication,
        }
    }
}

impl<G,R> TransformInput<G,R,R>
where
    G: Curve<R>,
    R: Real,
{
    /// Transfrom an input such that the wrapped generator changes its domain from [0.0,1.0] to
    /// the domain wished for.
    pub fn normalized_to_domain(generator: G, start: R, end: R) -> Self {
        Self::new(generator, -start, (end - start).recip())
    }
}

impl<G,A,M,I> Generator<I> for TransformInput<G,A,M>
where
    I: Mul<M>,
    I::Output: Add<A>,
    A: Copy,
    M: Copy,
    G: Generator<<<I as Mul<M>>::Output as Add<A>>::Output>,
{
    type Output = G::Output;
    fn gen(&self, input: I) -> Self::Output {
        self.inner.gen(input * self.multiplication + self.addition)
    }
}

impl<G,A,M,I> Interpolation<I> for TransformInput<G,A,M>
where
    I: Mul<M>,
    I::Output: Add<A>,
    A: Copy,
    M: Copy,
    G: Interpolation<<<I as Mul<M>>::Output as Add<A>>::Output>,
{}

impl<G,R> Curve<R> for TransformInput<G,R,R>
where
    G: Curve<R>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        let orig = self.inner.domain();
        let start = (orig[0] - self.addition) / self.multiplication;
        let end = (orig[1] - self.addition) / self.multiplication;
        [start,end]
    }
}

/// Struct which chains two Interpolation together to one Interpolation.
///
/// This `struct` is created by [`Interpolation::chain`]. See its documentation for more.
#[derive(Clone, Debug)]
pub struct Chain<A,B>{
    first: A,
    second: B
}

impl<A,B,T> Generator<T> for Chain<A,B>
where
    A: Interpolation<T>,
    B: Interpolation<A::Output>
{
    type Output = B::Output;
    fn gen(&self, scalar: T) -> Self::Output {
        self.second.gen(self.first.gen(scalar))
    }
}

impl<A,B,T> Interpolation<T> for Chain<A,B>
where
    A: Interpolation<T>,
    B: Interpolation<A::Output>
{}

impl<A,B,R> Curve<R> for Chain<A,B>
where
    A: Curve<R>,
    B: Interpolation<A::Output>,
    R: Real,
{
    fn domain(&self) -> [R; 2] {
        self.first.domain()
    }
}
