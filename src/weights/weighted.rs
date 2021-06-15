//! The adaptor `Weighted` can be used for all interpolations to hide the inner workings of a weighted element.

use core::marker::PhantomData;
use core::ops::Div;
use num_traits::real::Real;
use crate::{Generator, Interpolation, Curve, Homogeneous};

/// Interpolation Adaptor used for weighted elements to automatically unwrap them from their weights.
///
/// This Adaptor is often appended to an interpolation with weighted elements to automatically unwrap them.
pub struct Weighted<G,T,R>{
    inner: G,
    _phantoms: (PhantomData<T>,PhantomData<R>),
}

impl<G,T,R> Weighted<G,T,R>{
    /// Use the `Weighted` Adaptor on the given weighted interpolation to automatically unwrap the elements of their weight.
    pub fn new(gen: G) -> Self {
        Weighted {
            inner: gen,
            _phantoms: (PhantomData, PhantomData),
        }
    }
    /// Return the inner interpolation.
    pub fn inner(self) -> G {
        self.inner
    }
}

impl<T,R,G,I> Generator<I> for Weighted<G,T,R>
where
    G: Generator<I, Output = Homogeneous<T,R>>,
    T: Div<R, Output = T>
{
    type Output = T;
    fn gen(&self, input: I) -> Self::Output {
        self.inner.gen(input).project()
    }
}

impl<T,R,G,I> Interpolation<I> for Weighted<G,T,R>
where
    G: Interpolation<I, Output = Homogeneous<T,R>>,
    T: Div<R, Output = T>
{}

impl<T,R,G> Curve<R> for Weighted<G,T,R>
where
    G: Curve<R, Output = Homogeneous<T,R>>,
    T: Div<R, Output = T>,
    R: Real
{
    fn domain(&self) -> [R; 2] {
        self.inner.domain()
    }
}
