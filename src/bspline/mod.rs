//! This implementation of BSpline has a performance of O(n + d^2), with n number of elements and
//! d degree of the curve. There are implementations with a performance of O(log n + d^2), however
//! they need to allocate memory on the heap. This implementation does not if one uses arrays as
//! the collection of the elements. We assume that most of the time this tradeoff pays off.
//! If you have a use case in which you have a bspline with a large number of elements,
//! don't hesitate to create an issue on github and tell us about it.
//! Another option is to divide the bspline into fewer pieces.

mod error;
mod builder;

pub use error::{BSplineError, NonValidDegree, TooSmallWorkspace, NotSorted, TooFewElements};
pub use builder::BSplineBuilder;

use crate::{Generator, SortedGenerator, DiscreteGenerator, Space, Interpolation, Curve, Merge};
use crate::builder::Unknown;
use builder::Open;
use num_traits::real::Real;

use core::fmt::Debug;

/// BSplines are generalisations of Bezier Curves.
/// They allow you to define curves with a lot of control points without increasing the degree of the curve.
/// In contrast to Bezier Curves, BSplines do have a locally property.
/// That is, changing one control points only affects a local area of the curve, not the whole curve.
#[derive(Debug, Copy, Clone)]
pub struct BSpline<K,E,S>
{
    elements: E,
    knots: K,
    space: S,
    degree: usize,
}

impl BSpline<Unknown, Unknown, Unknown>{
    /// Return a builder instance of BSpline.
    pub fn builder() -> BSplineBuilder<Unknown, Unknown, Unknown, Unknown, Open>{
        BSplineBuilder::new()
    }
}

impl<K,E,S> BSpline<K,E,S>
where
    E: DiscreteGenerator,
    S: Space<E::Output>,
{
    /// Creates a workspace and copies degree+1 elements into it, starting from given index.
    fn workspace(&self, index: usize) -> impl AsMut<[E::Output]>{
        let mut workspace = self.space.workspace();
        let mut_workspace = workspace.as_mut();
        for (i,val) in mut_workspace.iter_mut().enumerate().take(self.degree + 1){
            *val = self.elements.gen(index-self.degree+i);
        }
        workspace
    }
}

impl<K,E,S,R> Generator<R> for BSpline<K,E,S>
where
    E: DiscreteGenerator,
    S: Space<E::Output>,
    E::Output: Merge<R> + Copy,
    R: Real + Debug,
    K: SortedGenerator<Output = R>
{
    type Output = E::Output;
    fn gen(&self, scalar: R) -> E::Output {
        // we do NOT calculaute a possible multiplicity of the scalar, as we assume
        // the chance of hitting a knot is almost zero.
        let lower_cut = self.degree;
        let upper_cut = self.knots.len() - self.degree;
        // The strict_upper_bound is easier to calculate and behaves nicely on the edges of the array.
        // Such it is more ergonomic than using upper_border.
        let index = self.knots.strict_upper_bound_clamped(scalar, lower_cut, upper_cut);

        //copy elements into workspace
        let mut workspace = self.workspace(index);
        let elements = workspace.as_mut();

        for r in 1..=self.degree {
            for j in 0..=(self.degree-r){
                let i = j+r+index-self.degree;
                let factor = (scalar - self.knots.gen(i-1))/(self.knots.gen(i+self.degree-r) - self.knots.gen(i-1));
                elements[j] = elements[j].merge(elements[j+1], factor);
            }
        }
        elements[0]
    }
}

impl<K,E,S,R> Interpolation<R> for BSpline<K,E,S>
where
    E: DiscreteGenerator,
    S: Space<E::Output>,
    E::Output: Merge<R> + Copy,
    R: Real + Debug,
    K: SortedGenerator<Output = R>
{}

impl<K,E,S,R> Curve<R> for BSpline<K,E,S>
where
    E: DiscreteGenerator,
    S: Space<E::Output>,
    E::Output: Merge<R> + Copy,
    R: Real + Debug,
    K: SortedGenerator<Output = R>
{
    fn domain(&self) -> [R; 2] {
        [self.knots.gen(self.degree-1), self.knots.gen(self.knots.len() - self.degree)]
    }
}

// impl<R,T> BSpline<R,Vec<T>,T,Vec<R>>
// {
//     /// Create a closed curve bspline which resembles a loop.
//     /// The number of elements and the number of knots have to be equal.
//     /// The domain is is the first and last knot given.
//     pub fn with_wrapping_knots<C>(collection: C, degree: usize) {
//         //TODO: clone the first control point and push it to the end
//         //TODO: clone the first degree+2 knots and push them also to the end
//     }
// }

// impl<R,E,T> BSpline<R,E,T,Vec<R>>
// where
//     E: AsRef<[T]>,
//     R: Real
// {
//     /// Create a bspline which touches its first and last control point
//     /// and has a domain of [0.0,1.0].
//     /// The degree of the curve is given by elements.len() - knots.len() - 1
//     pub fn with_clamped_ends<K>(elements: E, knots: K) -> Self
//     where
//         K: AsRef<[R]>
//     {
//         let elem_len = elements.as_ref().len();
//         let knots_len = knots.as_ref().len();
//         assert!(elem_len > knots_len +1);
//         let degree = elem_len - knots_len - 1;
//         let mut vec = Vec::with_capacity(knots_len + 2*degree + 2);
//         for _ in 0..degree+1{
//             vec.push(R::zero());
//         }
//         vec.extend(knots.as_ref());
//         for _ in 0..degree+1{
//             vec.push(R::one());
//         }
//         BSpline {
//             elements,
//             knots: vec,
//             degree,
//             _phantoms: (PhantomData, PhantomData)
//         }
//     }
// }

impl<K,E,S> BSpline<K,E,S>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    S: Space<E::Output>,
{
    /// Creates a bspline curve of elements and knots given.
    /// The resulting degree of the curve is elements.len() - knots.len() +1
    /// The degree has to be at least 1.
    /// The knots should be sorted.
    /// The domain for the curve with degree p is knots[p-1] and knots[knots.len() - p -2].
    pub fn new(elements: E, knots: K, space: S) -> Result<Self, BSplineError>
    {
        //Test if we have at least two elements
        if elements.len() < 2{
            return Err(TooFewElements::new(elements.len()).into());
        }
        // Test if degree is strict positive
        if knots.len() < elements.len() {
            return Err(NonValidDegree::new(knots.len() as isize - elements.len() as isize +1).into());
        }
        // Test if we have enough elements for the degree
        if elements.len() < knots.len() - elements.len() {
            return Err(TooFewElements::new(elements.len()).into());
        }
        let degree = knots.len() - elements.len() + 1;
        if space.len() <= degree {
            return Err(TooSmallWorkspace::new(space.len(),degree).into());
        }
        Ok(BSpline {
            elements,
            knots,
            degree,
            space,
        })
    }
}

impl<K,E,S> BSpline<K,E,S>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    S: Space<E::Output>,
{
    /// Creates a bspline curve of elements and knots given.
    /// The resulting degree of the curve is elements.len() - knots.len() + 1
    /// The degree has to be at least 1.
    /// The knots should be sorted.
    /// The domain for the curve with degree p is knots[p-1] and knots[knots.len() - p -2].
    pub fn new_unchecked(elements: E, knots: K, space: S) -> Self
    {
        let degree = knots.len() - elements.len() + 1;
        BSpline {
            elements,
            knots,
            space,
            degree,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn linear_bspline() {
        let expect = [(-1.0,-1.0),(0.0, 0.0), (0.2, 0.2), (0.4, 0.4), (0.6, 0.6),
                          (0.8, 0.8), (1.0, 1.0),(2.0,2.0)];
        let points = [0.0f32, 1.0];
        let knots = [0.0f32, 1.0];
        let spline = BSpline::builder()
            .elements(points)
            .knots(knots)
            .constant::<2>()
            .build().unwrap();
        for i in 0..expect.len(){
            assert_f32_near!(spline.gen(expect[i].0),expect[i].1);
        }
    }
    #[test]
    fn quadratic_bspline() {
        let expect = [(0.0, 0.0), (0.5, 0.125), (1.0, 0.5), (1.4, 0.74), (1.5, 0.75),
                          (1.6, 0.74), (2.0, 0.5), (2.5, 0.125), (3.0, 0.0)];
        let points = [0.0f32, 0.0, 1.0, 0.0, 0.0];
        let knots = [0.0f32, 0.0, 1.0, 2.0, 3.0, 3.0];
        let spline = BSpline::builder()
            .elements(points)
            .knots(knots)
            .constant::<3>()
            .build().unwrap();
        for i in 0..expect.len(){
            assert_f32_near!(spline.gen(expect[i].0),expect[i].1);
        }
    }
    #[test]
    fn cubic_bspline() {
        let expect = [(-2.0, 0.0), (-1.5, 0.125), (-1.0, 1.0), (-0.6, 2.488),
                           (0.0, 4.0), (0.5, 2.875), (1.5, 0.12500001), (2.0, 0.0)];
        let points = [0.0f32, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0];
        let knots = [-2.0f32, -2.0, -2.0, -1.0, 0.0, 1.0, 2.0, 2.0, 2.0];
        let spline = BSpline::builder()
            .elements(points)
            .knots(knots)
            .constant::<4>()
            .build().unwrap();
        for i in 0..expect.len(){
            assert_f32_near!(spline.gen(expect[i].0),expect[i].1);
        }
    }
    #[test]
    fn quartic_bspline() {
        let expect = [(0.0, 0.0), (0.4, 0.0010666668), (1.0, 0.041666668),
                          (1.5, 0.19791667), (2.0, 0.4583333), (2.5, 0.5989583),
                          (3.0, 0.4583333), (3.2, 0.35206667), (4.1, 0.02733751),
                          (4.5, 0.002604167), (5.0, 0.0)];
        let points = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let knots = [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0];
        let spline = BSpline::builder()
            .elements(points)
            .knots(knots)
            .constant::<5>()
            .build().unwrap();
        for i in 0..expect.len(){
            assert_f32_near!(spline.gen(expect[i].0),expect[i].1);
        }
    }
    #[test]
    fn quartic_bspline_f64() {
        let expect = [(0.0, 0.0), (0.4, 0.001066666666666667), (1.0, 0.041666666666666664),
                                           (1.5, 0.19791666666666666), (2.0, 0.45833333333333337), (2.5, 0.5989583333333334),
                                           (3.0, 0.4583333333333333), (3.2, 0.3520666666666666), (4.1, 0.027337500000000046),
                                           (4.5, 0.002604166666666666), (5.0, 0.0)];
        let points = [0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0];
        let knots = [0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 5.0, 5.0];
        let spline = BSpline::builder()
            .elements(points)
            .knots(knots)
            .constant::<5>()
            .build().unwrap();
        for i in 0..expect.len(){
            assert_f64_near!(spline.gen(expect[i].0),expect[i].1);
        }
    }
}
