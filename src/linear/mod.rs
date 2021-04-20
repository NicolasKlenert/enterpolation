// TODO: creation of Interpolations should not panic, instead it should return a Result!

pub mod linear_equidistant;

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use crate::{Interpolation, Stepper, EnterpolationError};

/// Find the indices in which the given element is in between.
/// We assume that the collection is non-empty and ordered, to use binary search.
/// If one or more elements in the collections are exactly equal to the element,
/// the function will return a border in which either index returned
/// will be the index of an element equal to the element given.
/// If the given element is smaller/bigger than every element in the collection, then
/// the borders given will the the smallest/biggest possible
fn find_borders<C,T>(collection: C, element: T) -> (usize, usize)
where
    C: AsRef<[T]>,
    T: PartialOrd + Copy
{
    let mut min_index = 0;
    let mut max_index = collection.as_ref().len() - 1;

    while max_index - min_index > 1 {
        let index = min_index + (max_index - min_index) / 2;
        let sample = collection.as_ref()[index];

        if element < sample {
            max_index = index;
        } else {
            min_index = index;
        }
    }
    (min_index, max_index)
}


/// Linear interpolate/extrapolate with the elements and knots given.
/// Knots should be in increasing order and there has to be at least 2 knots.
/// Also there has to be the same amount of elements and knots.
/// These constrains are not checked!
pub fn linear<E,T,K>(elements: E, knots: K, scalar: f64) -> T
where
    E: AsRef<[T]>,
    K: AsRef<[f64]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let (min_index, max_index) = find_borders(&knots, scalar);
    let min = knots.as_ref()[min_index];
    let max = knots.as_ref()[max_index];
    let min_point = elements.as_ref()[min_index];
    let max_point = elements.as_ref()[max_index];
    let factor = (scalar - min) / (max - min);
    min_point * (1.0 - factor) + max_point * factor
}

pub struct Linear<E,T,K>
where
    E: AsRef<[T]>,
    K: AsRef<[f64]>,
    T: Add<Output = T> + Mul<f64, Output = T>
{
    elements: E,
    knots: K,
    element: PhantomData<T>
}

impl<E,T,K> Interpolation for Linear<E,T,K>
where
    E: AsRef<[T]>,
    K: AsRef<[f64]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    type Output = T;
    fn get(&self, scalar: f64) -> T {
        linear(&self.elements, &self.knots, scalar)
    }
}

impl<T> Linear<Vec<T>,T,Vec<f64>>
where
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
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
            element: PhantomData
        })
    }

    /// Create a linear interpolation with at least 2 elements.
    /// Knots are calculated with the given closure, which takes the index and the reference to the element.
    /// Knots should be in increasing order. This is not checked.
    /// For a constant speed of the curve, the distance between the elements should be used.
    pub fn from_collection_with<C,F>(collection: C, func: F) -> Result<Self, EnterpolationError>
    where
        C: IntoIterator<Item = T>,
        F: FnMut((usize,&T)) -> f64,
    {
        let elements: Vec<T> = collection.into_iter().collect();
        if elements.len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 2,
            });
        }
        let knots: Vec<f64> = elements.iter().enumerate().map(func).collect();
        Ok(Linear {
            elements,
            knots,
            element: PhantomData
        })
    }

    /// Create a linear interpolation of the elements with given knots.
    /// Knots should be in increasing order and there has to be at least 2 elements.
    /// The increasing order of knots is not checked.
    pub fn from_collection_with_knots<C>(collection: C) -> Result<Self, EnterpolationError>
    where C: IntoIterator<Item = (T, f64)>
    {
        let iter = collection.into_iter();
        let mut elements: Vec<T> = Vec::with_capacity(iter.size_hint().0);
        let mut knots: Vec<f64> = Vec::with_capacity(iter.size_hint().0);
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
            element: PhantomData
        })
    }
}

impl<P,T,K> Linear<P,T,K>
where
    P: AsRef<[T]>,
    K: AsRef<[f64]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
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
            element: PhantomData
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Interpolation;

    #[test]
    fn linear() {
        let lin = Linear::from_collection(vec![20.0,100.0,0.0,200.0]).unwrap();
        let mut iter = lin.take(7);
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        for i in 0..=6 {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn extrapolation() {
        //TODO: test non-uniform domain and extrapolation
        let lin = Linear::from_collection_with_knots(vec![(20.0,1.0),(100.0,2.0),(0.0,3.0),(200.0,4.0)]).unwrap();
        assert_f64_near!(lin.get(1.5), 60.0);
        assert_f64_near!(lin.get(2.5), 50.0);
        assert_f64_near!(lin.get(-1.0), -140.0);
        assert_f64_near!(lin.get(5.0), 400.0);

    }

    #[test]
    fn find_borders() {
        let arr = [0.0,0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1.0];
        assert_eq!(super::find_borders(&arr,0.35),(3,4));
        assert_eq!(super::find_borders(&arr,-1.0),(0,1));
        assert_eq!(super::find_borders(&arr,20.0),(9,10));
        // test if element given es equal to a knot
        let borders = super::find_borders(&arr,0.5);
        assert!(borders.0 == 5 || borders.1 == 5);
        // test if find_borders works with only 1 element
        let arr = [5.0];
        assert_eq!(super::find_borders(&arr,0.5), (0,0));
    }
}
