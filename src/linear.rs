// TODO: creation of Interpolations should not panic, instead it should return a Result!

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use crate::{Interpolation, find_borders, Stepper};

/// Linear interpolate/extrapolate with the points and knots given.
/// Knots should be in increasing order and there has to be at least 2 knots.
/// Also there has to be the same amount of points and knots.
/// These constrains are not checked!
pub fn linear<P,T,K>(points: P, knots: K, scalar: f64) -> T
where
    P: AsRef<[T]>,
    K: AsRef<[f64]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let (min_index, max_index) = find_borders(&knots, scalar);
    let min = knots.as_ref()[min_index];
    let max = knots.as_ref()[max_index];
    let min_point = points.as_ref()[min_index];
    let max_point = points.as_ref()[max_index];
    let factor = (scalar - min) / (max - min);
    min_point * (1.0 - factor) + max_point * factor
}

pub struct Linear<P,T,K>
where
    P: AsRef<[T]>,
    K: AsRef<[f64]>,
    T: Add<Output = T> + Mul<f64, Output = T>
{
    points: P,
    knots: K,
    element: PhantomData<T>
}

impl<P,T,K> Interpolation for Linear<P,T,K>
where
    P: AsRef<[T]>,
    K: AsRef<[f64]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    type Output = T;
    fn get(&self, scalar: f64) -> T {
        linear(&self.points, &self.knots, scalar)
    }
}

impl<T> Linear<Vec<T>,T,Vec<f64>>
where
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    /// Creates a linear interpolation of the points given with equidistant knots.
    /// There has to be at least 2 elements.
    pub fn new<C>(collection: C) -> Self
    where C: IntoIterator<Item = T>
    {
        let points: Vec<T> = collection.into_iter().collect();
        assert!(points.len() > 1);
        Linear {
            knots: Stepper::new(points.len()).into_iter().collect(),
            points,
            element: PhantomData
        }
    }

    /// Create a linear interpolation with the points and knots given.
    /// Knots should be between 0.0 and 1.0 and in increasing order and there has to be at least 2 elements.
    pub fn with_knots<C>(collection: C) -> Self
    where C: IntoIterator<Item = (T, f64)>
    {
        let iter = collection.into_iter();
        let mut points: Vec<T> = Vec::with_capacity(iter.size_hint().0);
        let mut knots: Vec<f64> = Vec::with_capacity(iter.size_hint().0);
        for (elem, knot) in iter {
            points.push(elem);
            knots.push(knot);
        }
        assert!(points.len() > 1);
        Linear {
            points,
            knots,
            element: PhantomData
        }
    }
}

impl<P,T,K> Linear<P,T,K>
where
    P: AsRef<[T]>,
    K: AsRef<[f64]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    /// Create a linear interpolation with slice-like collections of points and knots.
    /// As well as the restriction to the knots explained in [with_knots](Linear::with_knots)
    /// there is also the restriction that the length of points and knots do have to be the same!
    pub fn with_raw(points: P, knots: K) -> Self
    {
        assert!(points.as_ref().len() > 1);
        assert_eq!(knots.as_ref().len(), points.as_ref().len());
        Linear {
            points,
            knots,
            element: PhantomData
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Interpolation;

    #[test]
    fn linear() {
        let lin = Linear::new(vec![20.0,100.0,0.0,200.0]);
        let mut iter = lin.take(7);
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        for i in 0..=6 {
            let val = iter.next().unwrap();
            assert!((val - expected[i]).abs() < 0.001, "linear: {}, should be: {}", val, expected[i]);
        }
    }

    #[test]
    fn extrapolation() {
        //TODO: test non-uniform domain and extrapolation
        let lin = Linear::new(vec![20.0,100.0,0.0,200.0]);
        //add https://docs.rs/assert_float_eq/1.1.3/assert_float_eq/

        // for i in 0..=6 {
        //     let val = iter.next().unwrap();
        //     assert!((val - expected[i]).abs() < 0.001, "linear: {}, should be: {}", val, expected[i]);
        // }
    }
}
