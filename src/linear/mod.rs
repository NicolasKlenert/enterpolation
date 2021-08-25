//! Linear and quasi-linear interpolations.
//!
//! The easist way to create a linear interpolation is by using the builder pattern of [`LinearBuilder`].
//!
//! ```rust
//! # use std::error::Error;
//! # use enterpolation::{linear::{Linear, LinearError}, Generator, Curve};
//! # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
//! #
//! # fn main() -> Result<(), LinearError> {
//! let linear = Linear::builder()
//!                 .elements([0.0,5.0,3.0])
//!                 .knots([0.0,1.0,2.0])
//!                 .build()?;
//! let results = [0.0,2.5,5.0,4.0,3.0];
//! for (value,result) in linear.take(5).zip(results.iter().copied()){
//!     assert_f64_near!(value, result);
//! }
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Linear interplations are one of the simplest forms of interpolations.
//! Most of the time, linear interpolations are used as an approximation of some smoother curve,
//! such they often have many elements.
//! For this reason the [`equidistant()`] method on the builder is recommended.
//!
//! `Linear` is always linear in its output but not necessarily in its input. In that case, we
//! say that the interpolation is quasi-linear.
//! One can imagine a linear interpolation between 2D points. Then quasi-linearity means that
//! the curve consists of lines between the given 2D points but its velocity may change non-linear.
//! To achieve a non-linear interpolation, the [`easing()`] method on the builder may be used.
//!
//! Linear equidistant constant interpolations are often wanted to define some specific curve
//! (like a specific gradient). To create such interpolation, the builder pattern can not be used yet.
//! Instead one should create a linear interpolation directly with its [`equidistant_unchecked()`] constructor.
//!
//! [linear module]: super
//! [`LinearBuilder`]: LinearBuilder
//! [plateus.rs]: https://github.com/NicolasKlenert/enterpolation/blob/main/examples/plateaus.rs
//! [`equidistant()`]: LinearBuilder::equidistant()
//! [`easing()`]: LinearBuilder::easing()
//! [`equidistant_unchecked()`]: Linear::equidistant_unchecked()

use crate::{Generator, Interpolation, Curve, SortedGenerator,
    DiscreteGenerator, ConstEquidistant, Easing, Identity};
use crate::builder::Unknown;
use num_traits::real::Real;
use topology_traits::Merge;

use core::fmt::Debug;

// mod hyper;
mod builder;
pub use builder::{LinearBuilder, LinearDirector};

pub mod error;
pub use error::{LinearError, TooFewElements, KnotElementInequality, NotSorted};

/// Linear Interpolation.
#[derive(Debug, Copy, Clone)]
pub struct Linear<K,E,F>
{
    elements: E,
    knots: K,
    easing: F,
}

impl Linear<Unknown,Unknown, Unknown> {
    /// Get the builder for a linear interpolation.
    ///
    /// The builder takes:
    /// - elements with `elements` or `elements_with_weights`
    /// - knots with either `knots` or `equidistant`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # use enterpolation::{linear::{Linear, LinearError}, Generator, Curve};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), LinearError> {
    /// let linear = Linear::builder()
    ///                 .elements([0.0,5.0,3.0])
    ///                 .equidistant::<f64>()
    ///                 .normalized()
    ///                 .build()?;
    /// let results = [0.0,2.5,5.0,4.0,3.0];
    /// for (value,result) in linear.take(5).zip(results.iter().copied()){
    ///     assert_f64_near!(value, result);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn builder() -> LinearBuilder<Unknown,Unknown, Identity, Unknown> {
        LinearBuilder::new()
    }
}

impl<R,K,E,F> Generator<R> for Linear<K,E,F>
where
    K: SortedGenerator<Output = R>,
    E: DiscreteGenerator,
    E::Output: Merge<R> + Debug,
    F: Easing<R, Output = R>,
    R: Real + Debug
{
    type Output = E::Output;
    /// # Panics
    ///
    /// Panics if `scalar` is NaN or similar.
    fn gen(&self, scalar: K::Output) -> Self::Output {
        //we use upper_border_with_factor as this allows us a performance improvement for equidistant knots
        let (min_index, max_index, factor) = self.knots.upper_border(scalar);
        let min_point = self.elements.gen(min_index);
        let max_point = self.elements.gen(max_index);
        min_point.merge(max_point,self.easing.gen(factor))
    }
}

impl<R,K,E,F> Interpolation<R> for Linear<K,E,F>
where
    K: SortedGenerator<Output = R>,
    E: DiscreteGenerator,
    E::Output: Merge<R> + Debug,
    F: Easing<R, Output = R>,
    R: Real + Debug
{}

impl<R,K,E,F> Curve<R> for Linear<K,E,F>
where
    K: SortedGenerator<Output = R>,
    E: DiscreteGenerator,
    E::Output: Merge<R> + Debug,
    F: Easing<R, Output = R>,
    R: Real + Debug
{
    fn domain(&self) -> [R; 2] {
        [self.knots.first().unwrap(), self.knots.last().unwrap()]
    }
}

impl<K,E,F> Linear<K,E,F>
where
    K: SortedGenerator,
    K::Output: Real,
    E: DiscreteGenerator,
    E::Output: Merge<K::Output>,
{
    /// Create a linear interpolation with slice-like collections of elements and knots.
    /// Knots should be in increasing order (not checked), there should be as many knots as elements
    /// and there has to be at least 2 elements.
    pub fn new(elements: E, knots: K, easing: F) -> Result<Self, LinearError>
    {
        if elements.len() < 2 {
            return Err(TooFewElements::new(elements.len()).into());
        }
        if knots.len() != elements.len() {
            return Err(KnotElementInequality::new(elements.len(), knots.len()).into());
        }
        Ok(Linear {
            elements,
            knots,
            easing,
        })
    }
}

impl<K,E,F> Linear<K,E,F>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    E::Output: Merge<K::Output>,
    K::Output: Real,
{
    /// Create a linear interpolation with slice-like collections of elements and knots.
    /// Knots should be in increasing order, there should be as many knots as elements
    /// and there has to be at least 2 elements.
    ///
    /// All requirements are not checked.
    pub fn new_unchecked(elements: E, knots: K, easing: F) -> Self
    {
        Linear {
            elements,
            knots,
            easing,
        }
    }
}

impl<R,T,const N: usize> Linear<ConstEquidistant<R,N>,[T;N], Identity>
{
    /// Create a linear interpolation with an array of elements.
    ///
    /// There has to be at least *two* elements, which is NOT checked.
    /// This function should be used if one wants to create a constant Interpolation.
    ///
    /// # Requirements
    ///
    /// The array has to be at least of length *two*.
    pub const fn equidistant_unchecked(elements: [T;N]) -> Self
    {
        Linear {
            elements,
            knots: ConstEquidistant::new(),
            easing: Identity::new(),
        }
    }
}

/// An array-allocated, const-creatable, linear interpolation with equidistant knot distribution.
///
/// This alias is used for convenience to help create constant curves.
///
/// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
pub type ConstEquidistantLinear<R,T,const N: usize> = Linear<ConstEquidistant<R,N>,[T;N], Identity>;


#[cfg(test)]
mod test {
    use super::*;
    use crate::Curve;

    #[test]
    fn linear_equidistant() {
        let lin = Linear::builder()
            .elements([20.0,100.0,0.0,200.0])
            .equidistant::<f64>()
            .normalized()
            .build().unwrap();
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = lin.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn linear() {
        //DynamicLinear
        let lin = Linear::builder()
            .elements([20.0,100.0,0.0,200.0])
            .knots([0.0,1.0/3.0,2.0/3.0,1.0])
            .build().unwrap();
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = lin.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn extrapolation() {
        let lin = Linear::builder()
            .elements([20.0,100.0,0.0,200.0])
            .knots([1.0,2.0,3.0,4.0])
            .build().unwrap();
        assert_f64_near!(lin.gen(1.5), 60.0);
        assert_f64_near!(lin.gen(2.5), 50.0);
        assert_f64_near!(lin.gen(-1.0), -140.0);
        assert_f64_near!(lin.gen(5.0), 400.0);
    }

    #[test]
    fn weights(){
        let lin = Linear::builder()
            .elements_with_weights([(0.0,9.0),(1.0,1.0)])
            .equidistant::<f64>()
            .normalized()
            .build().unwrap();
        assert_f64_near!(lin.gen(0.5), 0.1);
        // const LIN : Linear<f64,f64,ConstEquidistant<f64>,CollectionWrapper<[f64;4],f64>> = Linear::new_equidistant_unchecked([20.0,100.0,0.0,200.0]);
    }

    #[test]
    fn const_creation(){
        const LIN : ConstEquidistantLinear<f64,f64,4> = ConstEquidistantLinear::equidistant_unchecked([20.0,100.0,0.0,200.0]);
        // const LIN : Linear<f64,f64,ConstEquidistant<f64>,CollectionWrapper<[f64;4],f64>> = Linear::new_equidistant_unchecked([20.0,100.0,0.0,200.0]);
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = LIN.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }
}
