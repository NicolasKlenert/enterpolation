// TODO: creation of Interpolations should not panic, instead it should return a Result!
// TODO: rational Bezier curves!

pub mod bezier_deriative;

use core::ops::{Add, Mul, Sub};
use core::marker::PhantomData;
use crate::Interpolation;

//TODO: one can use bezier as static function (with const generics and AsMut<[T;N]>)
//TODO: and we do not have to mutate the input and do the copying in the function

/// Bezier curve interpolate/extrapolate with the elements given.
/// This mutates the elements, such copying them first is necessary!
pub fn bezier<P,T>(mut elements: P, scalar: f64) -> T
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let elements = elements.as_mut();
    let len = elements.len();
    for k in 1..len {
        for i in 0..len-k {
            elements[i] = elements[i]*(1.0-scalar)+elements[i+1]*scalar;
        }
    }
    elements[0]
}

/// Bezier curve interpolate/extrapolate and tangent calculation with the elements given.
/// This mutates the elements, such copying them first is necessary!
pub fn bezier_with_tangent<P,T>(mut elements: P, scalar: f64) -> (T,T)
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Sub<Output = T> + Copy
{
    let elements = elements.as_mut();
    let len = elements.len();
    for k in 1..len-1 {
        for i in 0..len-k {
            elements[i] = elements[i]*(1.0-scalar)+elements[i+1]*scalar;
        }
    }
    let tangent = (elements[1] - elements[0]) * len as f64;
    let result = elements[0]*(1.0-scalar)+elements[1]*scalar;
    (result, tangent)
}

pub struct Bezier<P,T>
where
    P: AsRef<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T>
{
    elements: P,
    element: PhantomData<T>
}

impl<P,T> Interpolation for Bezier<P,T>
where
    P: AsRef<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    type Output = T;
    fn get(&self, scalar: f64) -> T {
        bezier(self.elements.as_ref().to_owned(), scalar)
    }
}

impl<T> Bezier<Vec<T>,T>
where
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    /// Creates a bezier curve of elements given.
    /// There has to be at least 1 element.
    pub fn new<C>(collection: C) -> Self
    where C: IntoIterator<Item = T>
    {
        let elements: Vec<T> = collection.into_iter().collect();
        assert!(elements.len() > 0);
        Bezier {
            elements,
            element: PhantomData
        }
    }
}

// impl<P,T,K> Linear<P,T,K>
// where
//     P: AsRef<[T]>,
//     K: AsRef<[f64]>,
//     T: Add<Output = T> + Mul<f64, Output = T> + Copy
// {
//     /// Create a linear interpolation with slice-like collections of points and knots.
//     /// As well as the restriction to the knots explained in [with_knots](Linear::with_knots)
//     /// there is also the restriction that the length of points and knots do have to be the same!
//     pub fn with_raw(points: P, knots: K) -> Self
//     {
//         assert!(points.as_ref().len() > 1);
//         assert_eq!(knots.as_ref().len(), points.as_ref().len());
//         Linear {
//             points,
//             knots,
//             element: PhantomData
//         }
//     }
// }
//
// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::Interpolation;
//
//     #[test]
//     fn linear() {
//         let lin = Linear::new(vec![20.0,100.0,0.0,200.0]);
//         let mut iter = lin.take(7);
//         let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
//         for i in 0..=6 {
//             let val = iter.next().unwrap();
//             assert_f64_near!(val, expected[i]);
//         }
//     }
//
//     #[test]
//     fn extrapolation() {
//         //TODO: test non-uniform domain and extrapolation
//         let lin = Linear::with_knots(vec![(20.0,1.0),(100.0,2.0),(0.0,3.0),(200.0,4.0)]);
//         assert_f64_near!(lin.get(1.5), 60.0);
//         assert_f64_near!(lin.get(2.5), 50.0);
//         assert_f64_near!(lin.get(-1.0), -140.0);
//         assert_f64_near!(lin.get(5.0), 400.0);
//         // for i in 0..=6 {
//         //     let val = iter.next().unwrap();
//         //     assert!((val - expected[i]).abs() < 0.001, "linear: {}, should be: {}", val, expected[i]);
//         // }
//     }
// }
