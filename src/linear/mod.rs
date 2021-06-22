//! Linear Interpolations.
//!
//! Linear Interplations are one of the simplest forms of interpolations.
//! Most of the time, Linear Interpolations are used as an approximation of curves, such
//! Linear Interpolations often do have many elements. For this reason
//! we supply the specialized LinearEquidistant Interplation, in which we assume that the distance
//! of an element to it's neighbors is constant. This increases performance, as the search for
//! the border elements to calculate the linear interpolation with can be found in O(1)
//! instead of O(log n) with n being the number of elements in the interpolation structure.

use core::ops::{Add, Mul};
use crate::{Generator, Interpolation, Curve, SortedGenerator,
    DiscreteGenerator, ConstEquidistant};
use crate::builder::Unknown;
use num_traits::real::Real;

use core::fmt::Debug;

// mod hyper;
mod builder;
pub use builder::LinearBuilder;

pub mod error;
pub use error::{LinearError, ToFewElements, KnotElementInequality, NotSorted};

/// Linear interpolate/extrapolate with the elements and knots given.
/// Knots should be in increasing order and there has to be at least 2 knots.
/// Also there has to be the same amount of elements and knots.
/// These constrains are not checked!
fn linear<R,K,E>(elements: &E, knots: &K, scalar: R) -> E::Output
where
    E: DiscreteGenerator,
    K: SortedGenerator<Output = R>,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy + Debug,
    R: Real + Debug
{
    //we use upper_border_with_factor as this allows us a performance improvement for equidistant knots
    let (min_index, max_index, factor) = knots.upper_border(scalar);
    let min_point = elements.gen(min_index);
    let max_point = elements.gen(max_index);
    min_point * (R::one() - factor) + max_point * factor
}

/// Linear Interpolation.
#[derive(Debug, Copy, Clone)]
pub struct Linear<K,E>
{
    elements: E,
    knots: K,
}

impl Linear<Unknown,Unknown> {
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
    /// # use enterpolation::{linear::Linear, Generator, Curve};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let linear = Linear::builder()
    ///                 .elements([0.0,5.0,3.0])?
    ///                 .equidistant::<f64>()
    ///                 .build();
    /// let results = [0.0,2.5,5.0,4.0,3.0];
    /// for (value,result) in linear.take(5).zip(results.iter().copied()){
    ///     assert_f64_near!(value, result);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    pub fn builder() -> LinearBuilder<Unknown,Unknown,Unknown> {
        LinearBuilder::new()
    }
}

impl<R,K,E> Generator<R> for Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator<Output = R>,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy + Debug,
    R: Real + Debug
{
    type Output = E::Output;
    /// # Panics
    ///
    /// Panics if `scalar` is NaN or similar.
    fn gen(&self, scalar: K::Output) -> Self::Output {
        linear(&self.elements, &self.knots, scalar)
    }
}

impl<R,K,E> Interpolation<R> for Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator<Output = R>,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy + Debug,
    R: Real + Debug
{}

impl<R,K,E> Curve<R> for Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator<Output = R>,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy + Debug,
    R: Real + Debug
{
    fn domain(&self) -> [R; 2] {
        [self.knots.first().unwrap(), self.knots.last().unwrap()]
    }
}

impl<K,E> Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    E::Output: Add<Output = E::Output> + Mul<K::Output, Output = E::Output> + Copy,
    K::Output: Real
{
    /// Create a linear interpolation with slice-like collections of elements and knots.
    /// Knots should be in increasing order (not checked), there should be as many knots as elements
    /// and there has to be at least 2 elements.
    pub fn new(elements: E, knots: K) -> Result<Self, LinearError>
    {
        if elements.len() < 2 {
            return Err(ToFewElements::new(elements.len()).into());
        }
        if knots.len() != elements.len() {
            return Err(KnotElementInequality::new(elements.len(), knots.len()).into());
        }
        Ok(Linear {
            elements,
            knots,
        })
    }
}

impl<K,E> Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    E::Output: Add<Output = E::Output> + Mul<K::Output, Output = E::Output> + Copy,
    K::Output: Real
{
    /// Create a linear interpolation with slice-like collections of elements and knots.
    /// Knots should be in increasing order, there should be as many knots as elements
    /// and there has to be at least 2 elements.
    ///
    /// All requirements are not checked.
    pub fn new_unchecked(elements: E, knots: K) -> Self
    {
        Linear {
            elements,
            knots,
        }
    }
}

impl<R,T,const N: usize> Linear<ConstEquidistant<R,N>,[T;N]>
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
        }
    }
}

/// An array-allocated, const-creatable, linear interpolation with equidistant knot distribution.
///
/// This alias is used for convenience to help create constant curves.
///
/// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
pub type ConstEquidistantLinear<R,T,const N: usize> = Linear<ConstEquidistant<R,N>,[T;N]>;


#[cfg(test)]
mod test {
    use super::*;
    use crate::Curve;

    #[test]
    fn linear_equidistant() {
        //DynamicEquidistantLinear
        let lin = Linear::builder()
            .elements(vec![20.0,100.0,0.0,200.0]).unwrap()
            .equidistant::<f64>()
            .build();
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
            .elements(vec![20.0,100.0,0.0,200.0]).unwrap()
            .knots(vec![0.0,1.0/3.0,2.0/3.0,1.0]).unwrap()
            .build();
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
            .elements([20.0,100.0,0.0,200.0]).unwrap()
            .knots([1.0,2.0,3.0,4.0]).unwrap()
            .build();
        assert_f64_near!(lin.gen(1.5), 60.0);
        assert_f64_near!(lin.gen(2.5), 50.0);
        assert_f64_near!(lin.gen(-1.0), -140.0);
        assert_f64_near!(lin.gen(5.0), 400.0);
    }

    #[test]
    fn weights(){
        let lin = Linear::builder()
            .elements_with_weights(vec![(0.0,9.0),(1.0,1.0)]).unwrap()
            .equidistant::<f64>()
            .build();
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
