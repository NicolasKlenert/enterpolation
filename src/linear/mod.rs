//! Linear Interpolations
//! These Interpolations can be stacked together to create multidimensional Interpolations
//! Linear Interplations are one of the simplest forms of interpolations.
//! Most of the time, Linear Interpolations are used as an approximation of curves, such
//! Linear Interpolations often do have many elements. For this reason
//! we supply the specialized LinearEquidistant Interplation, in which we assume that the distance
//! of an element to it's neighbors is constant. This increases performance, as the search for
//! the border elements to calculate the linear interpolation with can be found in O(1)
//! instead of O(log n) with n being the number of elements in the interpolation structure.

// TODO: creation of Interpolations should not panic, instead it should return a Result!

pub mod linear_equidistant;

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use crate::{Generator, Interpolation, Curve, Stepper, EnterpolationError};
use crate::real::Real;
use crate::utils::upper_border;
use num_traits::cast::FromPrimitive;

/// Linear interpolate/extrapolate with the elements and knots given.
/// Knots should be in increasing order and there has to be at least 2 knots.
/// Also there has to be the same amount of elements and knots.
/// These constrains are not checked!
pub fn linear<R,E,T,K>(elements: E, knots: K, scalar: R) -> T
where
    E: AsRef<[T]>,
    K: AsRef<[R]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{
    let (min_index, max_index) = upper_border(knots.as_ref(), scalar);
    let min = knots.as_ref()[min_index];
    let max = knots.as_ref()[max_index];
    let min_point = elements.as_ref()[min_index];
    let max_point = elements.as_ref()[max_index];
    let factor = (scalar - min) / (max - min);
    min_point * (R::one() - factor) + max_point * factor
}

/// Linear Interpolation Structure with knots
/// If knots are roughly or exactly equidistant, consider using LinearEquidistant instead.
pub struct Linear<R,E,T,K>
{
    elements: E,
    knots: K,
    _phantoms: (PhantomData<R>, PhantomData<T>)
}

impl<R,E,T,K> Generator<R> for Linear<R,E,T,K>
where
    E: AsRef<[T]>,
    K: AsRef<[R]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{
    type Output = T;
    fn get(&self, scalar: R) -> T {
        linear(&self.elements, &self.knots, scalar)
    }
}

impl<R,E,T,K> Interpolation<R> for Linear<R,E,T,K>
where
    E: AsRef<[T]>,
    K: AsRef<[R]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{}

impl<R,E,T,K> Curve<R> for Linear<R,E,T,K>
where
    E: AsRef<[T]>,
    K: AsRef<[R]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{
    fn domain(&self) -> [R; 2] {
        [*self.knots.as_ref().first().unwrap(), *self.knots.as_ref().last().unwrap()]
    }
}

impl<R,T> Linear<R,Vec<T>,T,Vec<R>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
    /// Creates a linear interpolation of elements given with equidistant knots.
    /// There has to be at least 2 elements.
    pub fn from_collection<C>(collection: C) -> Result<Self, EnterpolationError>
    where C: IntoIterator<Item = T>
    {
        let elements: Vec<T> = collection.into_iter().collect();
        if elements.len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 2,
            });
        }
        Ok(Linear {
            knots: Stepper::new(elements.len()).into_iter().collect(),
            elements,
            _phantoms: (PhantomData, PhantomData)
        })
    }

    /// Create a linear interpolation with at least 2 elements.
    /// Knots are calculated with the given closure, which takes the index and the reference to the element.
    /// Knots should be in increasing order. This is not checked.
    /// For a constant speed of the curve, the distance between the elements should be used.
    pub fn from_collection_with<C,F>(collection: C, func: F) -> Result<Self, EnterpolationError>
    where
        C: IntoIterator<Item = T>,
        F: FnMut((usize,&T)) -> R,
    {
        let elements: Vec<T> = collection.into_iter().collect();
        if elements.len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 2,
            });
        }
        let knots: Vec<R> = elements.iter().enumerate().map(func).collect();
        Ok(Linear {
            elements,
            knots,
            _phantoms: (PhantomData, PhantomData)
        })
    }

    /// Create a linear interpolation of the elements with given knots.
    /// Knots should be in increasing order and there has to be at least 2 elements.
    /// The increasing order of knots is not checked.
    pub fn from_collection_with_knots<C>(collection: C) -> Result<Self, EnterpolationError>
    where C: IntoIterator<Item = (T, R)>
    {
        let iter = collection.into_iter();
        let mut elements: Vec<T> = Vec::with_capacity(iter.size_hint().0);
        let mut knots: Vec<R> = Vec::with_capacity(iter.size_hint().0);
        for (elem, knot) in iter {
            elements.push(elem);
            knots.push(knot);
        }
        if elements.len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 2,
            });
        }
        Ok(Linear {
            elements,
            knots,
            _phantoms: (PhantomData, PhantomData)
        })
    }
}

impl<R,P,T,K> Linear<R,P,T,K>
where
    P: AsRef<[T]>,
    K: AsRef<[R]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{
    /// Create a linear interpolation with slice-like collections of elements and knots.
    /// Knots should be in increasing order (not checked), there should be as many knots as elements
    /// and there has to be at least 2 elements.
    pub fn new(elements: P, knots: K) -> Result<Self, EnterpolationError>
    {
        if elements.as_ref().len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.as_ref().len(),
                expected: 2,
            });
        }
        if knots.as_ref().len() != elements.as_ref().len() {
            return Err(EnterpolationError::InvalidNumberKnots{
                name: "Linear".to_string(),
                found: knots.as_ref().len(),
                expected: "same amount as elements".to_string(),
            });
        }
        Ok(Linear {
            elements,
            knots,
            _phantoms: (PhantomData, PhantomData)
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Curve;

    #[test]
    fn linear() {
        let lin = Linear::<f64,_,_,_>::from_collection(vec![20.0,100.0,0.0,200.0]).unwrap();
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = lin.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn extrapolation() {
        let lin = Linear::from_collection_with_knots(vec![(20.0,1.0),(100.0,2.0),(0.0,3.0),(200.0,4.0)]).unwrap();
        assert_f64_near!(lin.get(1.5), 60.0);
        assert_f64_near!(lin.get(2.5), 50.0);
        assert_f64_near!(lin.get(-1.0), -140.0);
        assert_f64_near!(lin.get(5.0), 400.0);

    }
}
