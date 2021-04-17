// TODO: creation of Interpolations should not panic, instead it should return a Result!
// TODO: create LinearEquidistant Interpolation from Bezier, when a constant speed is wished for
// TODO: -> see https://www.researchgate.net/post/How-can-I-assign-a-specific-velocity-to-a-point-moving-on-a-Bezier-curve

use core::ops::{Add, Mul, Sub};
use core::marker::PhantomData;
use crate::MutInterpolation;

//TODO: one can use bezier as static function (with const generics and AsMut<[T;N]>)
//TODO: and we do not have to mutate the input and do the copying in the function

/// Struct used to effeciently calculate bezier curve and any deriative afterwards
pub struct BezierDeriative<P,T>
{
    /// the array to do calculations on. This has to be at least of len n(n+1), where n is the degree of the bezier
    /// (one less than the number of elements given)
    calculation: P,
    element: PhantomData<T>,
    degree: usize,
}

impl<P,T> MutInterpolation for BezierDeriative<P,T>
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    type Output = T;
    fn get(&mut self, scalar: f64) -> T {
        let mut counter = 0;
        for k in (1..self.degree).rev(){
            for _ in 0..k{
                self.calculation.as_mut()[counter + k + 1] = self.calculation.as_mut()[counter]*(1.0-scalar)+self.calculation.as_mut()[counter+1];
                counter += 1;
            }
            counter +=1;
        }
        self.calculation.as_mut()[counter]
    }
}

impl<P,T> BezierDeriative<P,T>
where
    P: AsRef<[T]>,
    T: Sub<Output = T> + Mul<f64, Output = T> + Copy
{
    /// Calculates the deriative of the given degree of the last point returned by get.
    /// This function panics if the wanted degree is bigger than the degree of the curve itself.
    pub fn deriative(&self, degree: usize) -> T {
        assert!(degree <= self.degree);
        let prod : usize = (self.degree-degree+1..=self.degree).product();
        let start = (degree+2..=self.degree+1).sum();
        let mut calc = self.calculation.as_ref()[start..=start+degree].to_owned();
        for k in 1..calc.len() {
            for i in 0..calc.len()-k {
                calc[i] = calc[i+1]-calc[i];
            }
        }
        calc[0] * prod as f64
    }
}

//
// impl<T> Linear<Vec<T>,T,Vec<f64>>
// where
//     T: Add<Output = T> + Mul<f64, Output = T> + Copy
// {
//     /// Creates a linear interpolation of elements given with equidistant knots.
//     /// There has to be at least 2 elements.
//     pub fn new<C>(collection: C) -> Self
//     where C: IntoIterator<Item = T>
//     {
//         let points: Vec<T> = collection.into_iter().collect();
//         assert!(points.len() > 1);
//         Linear {
//             knots: Stepper::new(points.len()).into_iter().collect(),
//             points,
//             element: PhantomData
//         }
//     }
//
//     /// Create a linear interpolation with at least 2 elements.
//     /// Knots are calculated with the given closure, which takes the index and the reference to the element.
//     /// For a constant speed of the curve, the distance between the elements should be used.
//     pub fn new_with<C,F>(collection: C, func: F) -> Self
//     where
//         C: IntoIterator<Item = T>,
//         F: FnMut((usize,&T)) -> f64,
//     {
//         let elements: Vec<T> = collection.into_iter().collect();
//         let knots: Vec<f64> = elements.iter().enumerate().map(func).collect();
//         Linear {
//             points: elements,
//             knots,
//             element: PhantomData
//         }
//     }
//
//     /// Create a linear interpolation of the elements with given knots.
//     /// Knots should be in increasing order and there has to be at least 2 elements.
//     pub fn with_knots<C>(collection: C) -> Self
//     where C: IntoIterator<Item = (T, f64)>
//     {
//         let iter = collection.into_iter();
//         let mut points: Vec<T> = Vec::with_capacity(iter.size_hint().0);
//         let mut knots: Vec<f64> = Vec::with_capacity(iter.size_hint().0);
//         for (elem, knot) in iter {
//             points.push(elem);
//             knots.push(knot);
//         }
//         assert!(points.len() > 1);
//         Linear {
//             points,
//             knots,
//             element: PhantomData
//         }
//     }
// }
//
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
