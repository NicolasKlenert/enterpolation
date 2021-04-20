// TODO: creation of Interpolations should not panic, instead it should return a Result!
// TODO: create LinearEquidistant Interpolation from Bezier, when a constant speed is wished for
// TODO: -> see https://www.researchgate.net/post/How-can-I-assign-a-specific-velocity-to-a-point-moving-on-a-Bezier-curve
// TODO: add elevate like in Bezier -> works exaclty the same, we only need to pass in slices...?! -> try to use bezier_elevate...

// traits which should be implemented are:
// P: AsRef<[T]> + AsMut<[T]>,
// T: Add<Output = T> + Sub<Output = T> + Mul<f64, Output = T> + Copy

use core::ops::{Add, Mul, Sub};
use core::iter::repeat;
use core::marker::PhantomData;
use crate::MutInterpolation;
use crate::utils::{triangle_folding, triangle_folding_inline};

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
        let last = triangle_folding(self.calculation.as_mut(), |first, second| first * (1.0-scalar) + second * scalar , self.degree);
        self.calculation.as_mut()[last]
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
        let len = calc.len();
        triangle_folding_inline(&mut calc, |first, second| second - first, len);
        calc[0] * prod as f64
    }
}

impl<P,T> BezierDeriative<P,T>
where
    P: AsRef<[T]>,
    T: Copy
{
    /// Divide the Bezier Curve to create two Bezier Curves which "cut" the current Bezier Curve
    /// at the last point returned by get.
    /// The two new Bezier Curves are pasted into the buffers given.
    /// Both buffer do have to be at least of length (1..degree+1).sum().
    pub fn divide_with_buffers<Q>(&self, mut first_buffer: Q, mut second_buffer: Q) -> (BezierDeriative<Q,T>, BezierDeriative<Q,T>)
    where Q: AsRef<[T]> + AsMut<[T]>
    {
        let mut counter = 0;
        let mut iteration_counter = 0;
        for k in (0..self.degree).rev(){
            first_buffer.as_mut()[iteration_counter] = self.calculation.as_ref()[counter];
            second_buffer.as_mut()[iteration_counter] = self.calculation.as_ref()[counter+k];
            counter += k+1;
            iteration_counter += 1;
        }
        (BezierDeriative::new(first_buffer, self.degree), BezierDeriative::new(second_buffer, self.degree))
    }
}

impl<P,T> BezierDeriative<P,T>
where
    P: AsMut<[T]>,
    T: Copy
{
    /// Divide the Bezier Curve to create two Bezier Curves at the last point returned by get.
    /// The Bezier Curve from [0.0,u] is pasted onto the current Bezier Curve, the other one is pasted onto the given buffer.
    /// The given buffer do have to be at least of length (1..degree+1).sum().
    pub fn divide_with_buffer_inline<Q>(&mut self, mut buffer: Q) -> BezierDeriative<Q,T>
    where Q: AsRef<[T]> + AsMut<[T]>
    {
        let mut counter = 0;
        let mut iteration_counter = 0;
        for k in (0..self.degree).rev(){
            self.calculation.as_mut()[iteration_counter] = self.calculation.as_mut()[counter];
            buffer.as_mut()[iteration_counter] = self.calculation.as_mut()[counter+k];
            counter += k+1;
            iteration_counter += 1;
        }
        BezierDeriative::new(buffer, self.degree)
    }
}

impl<T> BezierDeriative<Vec<T>,T>
where T: Copy
{
    /// Divide the Bezier Curve to create two Bezier Curves at the last point returned by get.
    pub fn divide(&self) -> (Self, Self)
    {
        let mut first_vec = self.calculation.clone();
        let mut second_vec = self.calculation.clone();
        let mut counter = 0;
        let mut iteration_counter = 0;
        for k in (0..self.degree).rev(){
            first_vec[iteration_counter] = self.calculation[counter];
            second_vec[iteration_counter] = self.calculation[counter+k];
            counter += k+1;
            iteration_counter += 1;
        }
        (BezierDeriative::new(first_vec, self.degree), BezierDeriative::new(second_vec, self.degree))
    }

    /// Divide the Bezier Curve to create two Bezier Curves at the last point returned by get.
    /// The Bezier Curve from [0.0,u] is pasted onto the current Bezier Curve.
    pub fn divide_inline(&mut self) -> Self {
        let mut vec = self.calculation.clone();
        let mut counter = 0;
        let mut iteration_counter = 0;
        for k in (0..self.degree).rev(){
            self.calculation[iteration_counter] = self.calculation[counter];
            vec[iteration_counter] = self.calculation[counter+k];
            counter += k+1;
            iteration_counter += 1;
        }
        BezierDeriative::new(vec, self.degree)
    }
}



impl<T> BezierDeriative<Vec<T>,T>
where T: Copy + Default
{
    /// Creates a bezier interpolation of elements given.
    /// There has to be at least 2 elements.
    pub fn from_collection<C>(collection: C) -> Self
    where C: IntoIterator<Item = T>
    {
        let mut calculation: Vec<T> = collection.into_iter().collect();
        let length = calculation.len();
        assert!(length > 1);
        calculation.extend(repeat(T::default()).take((1..length).sum()));
        BezierDeriative {
            calculation,
            degree: length - 1,
            element: PhantomData
        }
    }
}

impl<P,T> BezierDeriative<P,T>
where
    P: AsRef<[T]>
{
    /// Creates a bezier interpolation with given calculation space.
    /// The first degree + 1 elements in the calculation space will be used as elements.
    /// The calculation space has to have at least a length of (1..degree+1).sum().
    /// There has to be at least 2 elements.
    pub fn new(calculation: P, degree: usize) -> Self
    {
        assert!(degree > 0);
        assert!(calculation.as_ref().len() >= (1..degree+1).sum());
        BezierDeriative {
            calculation,
            degree,
            element: PhantomData
        }
    }
}
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
