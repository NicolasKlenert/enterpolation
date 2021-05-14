//! This implementation of BSpline has a performance of O(n + d^2), with n number of elements and
//! d degree of the curve. There are implementations with a performance of O(log n + d^2), however
//! they need to allocate memory on the heap. This implementation does not if one uses arrays as
//! the collection of the elements. We assume that most of the time this tradeoff pays off.
//! If you have a use case in which you have a bspline with a large number of elements,
//! don't hesitate to create an issue on github and tell us about it.
//! Another option is to divide the bspline into fewer pieces.

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use crate::{Interpolation, Curve};
use crate::utils::strict_upper_bound;
use crate::real::Real;

/// BSpline curve interpolate/extrapolate with the elements given. (De Boors Algorithm)
/// This mutates the elements, such copying them first is necessary!
/// Panics if not at least 1 element exists.
//TODO: think about what happens if we are out of range!
pub fn bspline<R,E,K,T>(mut elements: E, knots: K, degree: usize, scalar: R) -> T
where
    E: AsMut<[T]>,
    K: AsRef<[R]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{
    // we do NOT calculaute a possible multiplicity of the scalar, as we assume
    // the chance of hitting a knot is almost zero.
    let lower_cut = degree;
    let upper_cut = knots.as_ref().len() - degree -1;
    // The strict_upper_bound is easier to calculate and behaves nicely on the edges of the array.
    // Such it is more ergonomic than using upper_border.
    let index = strict_upper_bound(&knots.as_ref()[lower_cut..upper_cut], scalar);
    //add the index offset back
    let index = index + lower_cut;
    let knots = knots.as_ref();
    let elements = elements.as_mut();
    for r in 1..=degree {
        for i in ((index+r-degree-1)..index).rev(){
        // for i in (max_index-degree+r-1)..(max_index-1-multiplicity){
            let factor = (scalar - knots[i])/(knots[i+degree-r+1] - knots[i]);
            elements[i] = elements[i-1] * (R::one() - factor) + elements[i] * factor;
        }
    }
    elements[index-1]
}

/// BSplines are generalisations of Bezier Curves.
/// They allow you to define curves with a lot of control points without increasing the degree of the curve.
/// In contrast to Bezier Curves, BSplines do have a locally property.
/// That is, changing one control points only affects a local area of the curve, not the whole curve.
pub struct BSpline<R,E,T,K>
{
    elements: E,
    knots: K,
    degree: usize,
    _phantoms: (PhantomData<R>, PhantomData<T>)
}

impl<R,E,T,K> Interpolation for BSpline<R,E,T,K>
where
    E: AsRef<[T]> + ToOwned,
    E::Owned: AsMut<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real,
    K: AsRef<[R]>
{
    type Input = R;
    type Output = T;
    fn get(&self, scalar: R) -> T {
        bspline(self.elements.to_owned(), self.knots.as_ref(), self.degree, scalar)
    }
}

impl<R,E,T,K> Curve for BSpline<R,E,T,K>
where
    E: AsRef<[T]> + ToOwned,
    E::Owned: AsMut<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real,
    K: AsRef<[R]>
{}

impl<R,T> BSpline<R,Vec<T>,T,Vec<R>>
{
    /// Creates a bezier curve of elements given.
    /// There has to be at least 2 elements.
    // pub fn with_collection<C>(collection: C) -> Self
    // where C: IntoIterator<Item = T>
    // {
    //     let elements: Vec<T> = collection.into_iter().collect();
    //     assert!(elements.len() > 1);
    //     BSpline {
    //         elements,
    //         _phantoms: (PhantomData, PhantomData)
    //     }
    // }

    /// Create a closed curve bspline which resembles a loop.
    /// The number of elements and the number of knots have to be equal.
    /// The domain is is the first and last knot given.
    pub fn with_wrapping_knots<C>(collection: C, degree: usize) {
        //TODO: clone the first control point and push it to the end
        //TODO: clone the first degree+2 knots and push them also to the end
    }
}

impl<R,E,T> BSpline<R,E,T,Vec<R>>
where
    E: AsRef<[T]>,
    R: Real
{
    /// Create a bspline which touches its first and last control point
    /// and has a domain of [0.0,1.0].
    /// The degree of the curve is given by elements.len() - knots.len() - 1
    pub fn with_clamped_ends<K>(elements: E, knots: K) -> Self
    where
        K: AsRef<[R]>
    {
        let elem_len = elements.as_ref().len();
        let knots_len = knots.as_ref().len();
        assert!(elem_len > knots_len +1);
        let degree = elem_len - knots_len - 1;
        let mut vec = Vec::with_capacity(knots_len + 2*degree + 2);
        for _ in 0..degree+1{
            vec.push(R::zero());
        }
        vec.extend(knots.as_ref());
        for _ in 0..degree+1{
            vec.push(R::one());
        }
        BSpline {
            elements,
            knots: vec,
            degree,
            _phantoms: (PhantomData, PhantomData)
        }
    }
}

impl<R,E,T,K> BSpline<R,E,T,K>
where
    E: AsRef<[T]>,
    K: AsRef<[R]>,
{
    /// Creates a bspline curve of elements and knots given.
    /// The resulting degree of the curve is elements.len() - knots.len() -1
    /// The degree has to be at least 1.
    /// The knots should be sorted.
    /// The domain for the curve with degree p is knots[p] and knots[knots.len() - p -1].
    pub fn new(elements: E, knots: K) -> Self
    {
        assert!(knots.as_ref().len() > elements.as_ref().len() + 1);
        let degree = knots.as_ref().len() - elements.as_ref().len() - 1;
        BSpline {
            elements,
            knots,
            degree,
            _phantoms: (PhantomData, PhantomData)
        }
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn linear_bspline() {
        let expect: Vec<(f32, f32)> = vec![(0.0, 0.0), (0.2, 0.2), (0.4, 0.4), (0.6, 0.6),
                          (0.8, 0.8), (1.0, 1.0)];
        let points = [0.0f32, 1.0];
        let knots = [0.0f32, 0.0, 1.0, 1.0];
        let spline = BSpline::new(points, knots);
        for i in 0..expect.len(){
            assert_f32_near!(spline.get(expect[i].0),expect[i].1);
        }
    }
    #[test]
    fn quadratic_bspline() {
        let expect: Vec<(f32, f32)> = vec![(0.0, 0.0), (0.5, 0.125), (1.0, 0.5), (1.4, 0.74), (1.5, 0.75),
                          (1.6, 0.74), (2.0, 0.5), (2.5, 0.125), (3.0, 0.0)];
        let points = [0.0f32, 0.0, 1.0, 0.0, 0.0];
        let knots = [0.0f32, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0];
        let spline = BSpline::new(points, knots);
        for i in 0..expect.len(){
            assert_f32_near!(spline.get(expect[i].0),expect[i].1);
        }
    }
    #[test]
    fn cubic_bspline() {
        let expect: Vec<(f32, f32)> = vec![(-2.0, 0.0), (-1.5, 0.125), (-1.0, 1.0), (-0.6, 2.488),
                           (0.0, 4.0), (0.5, 2.875), (1.5, 0.12500001), (2.0, 0.0)];
        let points = [0.0f32, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0];
        let knots = [-2.0f32, -2.0, -2.0, -2.0, -1.0, 0.0, 1.0, 2.0, 2.0, 2.0, 2.0];
        let spline = BSpline::new(points, knots);
        for i in 0..expect.len(){
            assert_f32_near!(spline.get(expect[i].0),expect[i].1);
        }
    }
    #[test]
    fn quartic_bspline() {
        let expect: Vec<(f32, f32)> = vec![(0.0, 0.0), (0.4, 0.0010666668), (1.0, 0.041666668),
                          (1.5, 0.19791667), (2.0, 0.4583333), (2.5, 0.5989583),
                          (3.0, 0.4583333), (3.2, 0.35206667), (4.1, 0.02733751),
                          (4.5, 0.002604167), (5.0, 0.0)];
        let points: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let knots: Vec<f32> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0];
        let spline = BSpline::new(points, knots);
        for i in 0..expect.len(){
            assert_f32_near!(spline.get(expect[i].0),expect[i].1);
        }
    }
    #[test]
    fn quartic_bspline_f64() {
        let expect: Vec<(f64, f64)> = vec![(0.0, 0.0), (0.4, 0.001066666666666667), (1.0, 0.041666666666666664),
                                           (1.5, 0.19791666666666666), (2.0, 0.45833333333333337), (2.5, 0.5989583333333334),
                                           (3.0, 0.4583333333333333), (3.2, 0.3520666666666666), (4.1, 0.027337500000000046),
                                           (4.5, 0.002604166666666666), (5.0, 0.0)];
        let points: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let knots: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0, 5.0];
        let spline = BSpline::new(points, knots);
        for i in 0..expect.len(){
            assert_f64_near!(spline.get(expect[i].0),expect[i].1);
        }
    }
}
