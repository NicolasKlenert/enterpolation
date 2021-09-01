//! Basis spline curves
//!
//! The easist way to create a bspline is by using the builder pattern of [`BSplineBuilder`].
//!
//! ```rust
//! # use std::error::Error;
//! # use enterpolation::{bspline::{BSpline, BSplineError}, Generator, Curve};
//! # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
//! #
//! # fn main() -> Result<(), BSplineError> {
//! let bspline = BSpline::builder()
//!                 .clamped()
//!                 .elements([0.0,5.0,3.0,10.0,7.0])
//!                 .equidistant::<f64>()
//!                 .degree(3)
//!                 .normalized()
//!                 .constant::<4>()
//!                 .build()?;
//! let results = [0.0,2.346,3.648,4.302,4.704,5.25,6.2,7.27,8.04,8.09,7.0];
//! for (value,result) in bspline.take(results.len()).zip(results.iter().copied()){
//!     assert_f64_near!(value, result);
//! }
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! BSplines can be seen as many bezier curves put together. They have most properties of bezier curves
//! but changing an element in a bspline only affects a local area of the curve,
//! not the whole curve, like it is in bezier curves.
//! BSplines allow you to define curves with a lot of control points without increasing the degree of the curve.
//!
//! [`BSplineBuilder`]: BSplineBuilder
mod error;
mod builder;
mod adaptors;

pub use error::{BSplineError, InvalidDegree, TooSmallWorkspace, NotSorted, TooFewElements};
pub use adaptors::{BorderBuffer, BorderDeletion};
pub use builder::{BSplineBuilder, BSplineDirector};

use crate::{Generator, SortedGenerator, DiscreteGenerator, Space, Curve};
use crate::builder::Unknown;
use builder::Open;
use num_traits::real::Real;
use topology_traits::Merge;

use core::fmt::Debug;

/// BSpline curve.
///
/// See [bspline module] for more information.
///
/// [bspline module]: self
#[derive(Debug, Copy, Clone)]
pub struct BSpline<K,E,S>
{
    elements: E,
    knots: K,
    space: S,
    degree: usize,
}

impl BSpline<Unknown, Unknown, Unknown>{
    /// Get a builder for bsplines.
    ///
    /// The builder takes:
    /// - a mode, either [`open()`], which is default, [`clamped()`] or [`legacy()`]
    /// - elements with [`elements()`] or [`elements_with_weights()`]
    /// - knots with [`knots()`] or [`equidistant()`]
    /// - the kind of workspace to use with [`dynamic()`], [`constant()`] or [`workspace()`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # use enterpolation::{bezier::{Bezier, BezierError}, Generator, Curve};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), BezierError> {
    /// let bez = Bezier::builder()
    ///     .elements([20.0,100.0,0.0,200.0])
    ///     .normalized::<f64>()
    ///     .constant()
    ///     .build()?;
    /// let mut iter = bez.take(5);
    /// let expected = [20.0,53.75,65.0,98.75,200.0];
    /// for i in 0..=4 {
    ///     let val = iter.next().unwrap();
    ///     assert_f64_near!(val, expected[i]);
    /// }
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// [`open()`]: BSplineBuilder::open()
    /// [`clamped()`]: BSplineBuilder::clamped()
    /// [`legacy()`]: BSplineBuilder::legacy()
    /// [`elements()`]: BSplineBuilder::elements()
    /// [`elements_with_weights()`]: BSplineBuilder::elements_with_weights()
    /// [`knots()`]: BSplineBuilder::knots()
    /// [`equidistant()`]: BSplineBuilder::equidistant()
    /// [`dynamic()`]: BSplineBuilder::dynamic()
    /// [`constant()`]: BSplineBuilder::constant()
    /// [`workspace()`]: BSplineBuilder::workspace()
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

impl<K,E,S> BSpline<K,E,S>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    S: Space<E::Output>,
{
    /// Creates a bspline curve of elements and knots given.
    ///
    /// The resulting degree of the curve is `elements.len() - knots.len() +1`.
    /// The domain for the curve with degree `p` is `knots[p-1]` and `knots[knots.len() - p -2]`.
    ///
    /// The knots have to be sorted.
    ///
    /// # Errors
    ///
    /// [`TooFewElements`] if there are less than two elements.
    /// [`InvalidDegree`] if degree is not at least 1 and at most the number of elements - 1.
    /// [`TooSmallWorkspace`] if the workspace is not bigger than the degree of the curve.
    ///
    /// [`TooFewElements`]: BSplineError
    /// [`InvalidDegree`]: BSplineError
    /// [`TooSmallWorkspace`]: BSplineError
    pub fn new(elements: E, knots: K, space: S) -> Result<Self, BSplineError>
    {
        //Test if we have at least two elements
        if elements.len() < 2{
            return Err(TooFewElements::new(elements.len()).into());
        }
        // Test if degree is strict positive
        if knots.len() < elements.len() {
            return Err(InvalidDegree::new(knots.len() as isize - elements.len() as isize +1).into());
        }
        // Test if we have enough elements for the degree
        if elements.len() < knots.len() - elements.len() {
            return Err(InvalidDegree::new(knots.len() as isize - elements.len() as isize +1).into());
        }
        let degree = knots.len() - elements.len() + 1;
        if space.len() <= degree {
            return Err(TooSmallWorkspace::new(space.len(),degree).into());
        }
        Ok(BSpline {
            elements,
            knots,
            space,
            degree,
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
    ///
    /// The resulting degree of the curve is `elements.len() - knots.len() + 1`.
    /// The domain for the curve with degree `p` is `knots[p-1]` and `knots[knots.len() - p -2]`.
    /// The knots have to be sorted.
    ///
    /// # Panics
    ///
    /// The degree has to be at least 1, otherwise the library may panic at any time.
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
