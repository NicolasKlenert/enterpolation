// TODO: creation of Interpolations should not panic, instead it should return a Result!

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use crate::Interpolation;

/// Linear interpolate/extrapolate with the elements, assuming they are all equally far from each other.
/// There should be at least 1 element!
pub fn linear<E,T>(elements: E, scalar: f64) -> T
where
    E: AsRef<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let elements = elements.as_ref();
    let scaled = scalar * (elements.len()-1) as f64;
    let min_index = (scaled.floor() as usize).max(0);
    let max_index = (scaled.ceil() as usize).min(elements.len()-1);
    let factor = scaled.fract();
    elements[min_index] * (1.0 - factor) + elements[max_index] * factor
}

/// Linear interpolation/extrapolation with elements, assumed to be equally far away and sorted.
/// Domain of the interpolation is [0.0,1.0].
pub struct LinearEquidistant<E,T>
where
    E: AsRef<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T>
{
    elements: E,
    element: PhantomData<T>
}

impl<E,T> Interpolation for LinearEquidistant<E,T>
where
    E: AsRef<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    type Output = T;
    fn get(&self, scalar: f64) -> T {
        linear(&self.elements, scalar)
    }
}

impl<T> LinearEquidistant<Vec<T>,T>
where
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    /// Creates a linear interpolation of elements, aussuming they are equidistant.
    /// There has to be at least 1 element.
    pub fn from_collection<C>(collection: C) -> Self
    where C: IntoIterator<Item = T>
    {
        let elements: Vec<T> = collection.into_iter().collect();
        assert!(elements.len() > 0);
        LinearEquidistant {
            elements,
            element: PhantomData
        }
    }
}

impl<P,T> LinearEquidistant<P,T>
where
    P: AsRef<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    /// Create a linear interpolation with slice-like collections of elements.
    /// There has to be at least 1 element.
    pub fn new(elements: P) -> Self
    {
        assert!(elements.as_ref().len() > 0);
        LinearEquidistant {
            elements,
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
        let lin = LinearEquidistant::new(vec![20.0,100.0,0.0,200.0]);
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
        let constant = LinearEquidistant::new(&arr[..]);
        assert_f64_near!(constant.get(-1.0), 5.0);
        assert_f64_near!(constant.get(10.0), 5.0);
        assert_f64_near!(constant.get(0.5), 5.0);
    }

}
