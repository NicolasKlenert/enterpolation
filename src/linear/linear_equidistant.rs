//! Special Linear Interpolation in which the knots are assumed to be equidistant.
//! This allows an performance boost and makes interpolation O(1) instead of O(log n) with n
//! being the number of elements saved in the linear interpolation.

// TODO: creation of Interpolations should not panic, instead it should return a Result!

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use crate::{Generator, Interpolation, Curve};
use crate::real::Real;
use num_traits::cast::FromPrimitive;

/// Linear interpolate/extrapolate with the elements, assuming they are all equally far from each other.
/// There should be at least 1 element!
pub fn linear<R,E,T>(elements: E, scalar: R) -> T
where
    E: AsRef<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive,
{
    let elements = elements.as_ref();
    let scaled = scalar * R::from_usize(elements.len()-1).unwrap();
    let min_index = scaled.floor().to_usize().unwrap().max(0);
    let max_index = scaled.ceil().to_usize().unwrap().min(elements.len()-1);
    let factor = scaled.fract();
    elements[min_index] * (R::one() - factor) + elements[max_index] * factor
}

/// Linear interpolation/extrapolation with elements, assumed to be equally far away and sorted.
/// Domain of the interpolation is [0.0,1.0].
pub struct LinearEquidistant<R,E,T>
{
    elements: E,
    _phantoms: (PhantomData<R>, PhantomData<T>)
}

impl<R,E,T> Generator<R> for LinearEquidistant<R,E,T>
where
    E: AsRef<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
    type Output = T;
    fn get(&self, scalar: R) -> T {
        linear(&self.elements, scalar)
    }
}

impl<R,E,T> Interpolation<R> for LinearEquidistant<R,E,T>
where
    E: AsRef<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
}

impl<R,E,T> Curve<R> for LinearEquidistant<R,E,T>
where
    E: AsRef<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
    /// Return the domain of the Curve, in this case just [0.0,1.0].
    fn domain(&self) -> [R; 2] {
        [R::zero(), R::one()]
    }
}

impl<R,T> LinearEquidistant<R,Vec<T>,T>
{
    /// Creates a linear interpolation of elements, aussuming they are equidistant.
    /// There has to be at least 1 element.
    pub fn from_collection<C>(collection: C) -> Self
    where C: IntoIterator<Item = T>
    {
        let elements: Vec<T> = collection.into_iter().collect();
        assert!(!elements.is_empty());
        LinearEquidistant {
            elements,
            _phantoms: (PhantomData, PhantomData)
        }
    }
}

impl<R,P,T> LinearEquidistant<R,P,T>
where
    P: AsRef<[T]>,
{
    /// Create a linear interpolation with slice-like collections of elements.
    /// There has to be at least 1 element.
    pub fn new(elements: P) -> Self
    {
        assert!(!elements.as_ref().is_empty());
        LinearEquidistant {
            elements,
            _phantoms: (PhantomData, PhantomData)
        }
    }
}

// N has to be at least one element!
impl<R,T, const N: usize> LinearEquidistant<R,[T;N],T>
{
    /// Create a linear interpolation with an array of elements.
    /// There has to be at least 1 element, which is NOT checked.
    /// Should be used if one wants to create a constant Interpolation
    pub const fn new_unchecked(elements: [T;N]) -> Self
    {
        LinearEquidistant {
            elements,
            _phantoms: (PhantomData, PhantomData)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Curve;

    #[test]
    fn linear() {
        let lin = LinearEquidistant::<f64,_,_>::new(vec![20.0,100.0,0.0,200.0]);
        let mut iter = lin.take(7);
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        for i in 0..=6 {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn constant(){
        let arr = [5.0];
        let constant = LinearEquidistant::<f64,_,_>::new(&arr[..]);
        assert_f64_near!(constant.get(-1.0), 5.0);
        assert_f64_near!(constant.get(10.0), 5.0);
        assert_f64_near!(constant.get(0.5), 5.0);
    }

    #[test]
    fn const_creation(){
        const LIN : LinearEquidistant<f64,[f64;4],f64> = LinearEquidistant::new_unchecked([20.0,100.0,0.0,200.0]);
        let mut iter = LIN.take(7);
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        for i in 0..=6 {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

}
