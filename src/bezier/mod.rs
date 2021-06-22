//! Bezier curves do have a performance of O(n^2), as their degree corresponds with the number of elements.

// TODO: create LinearEquidistant Interpolation from Bezier, when a constant speed is wished for
// TODO: -> see https://www.researchgate.net/post/How-can-I-assign-a-specific-velocity-to-a-point-moving-on-a-Bezier-curve
use core::ops::{Add, Mul, Sub, IndexMut};
use crate::{Generator, Interpolation, Curve, Stepper, Space, DiscreteGenerator};
use crate::utils::{triangle_folding_inline, lower_triangle_folding_inline};
use crate::builder::Unknown;
use num_traits::real::Real;
use num_traits::cast::FromPrimitive;
use core::marker::PhantomData;

mod builder;
pub use builder::BezierBuilder;
mod error;
pub use error::{BezierError, Empty, TooSmallWorkspace};

/// Bezier curve interpolate/extrapolate with the elements given.
/// This mutates the elements, such copying them first is necessary!
/// Panics if not at least 1 element exists.
fn bezier<R,P,T>(mut elements: P, scalar: R) -> T
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{
    let len = elements.as_mut().len();
    triangle_folding_inline(elements.as_mut(), |first, second| first * (R::one()-scalar) + second * scalar , len - 1);
    elements.as_mut()[0]
}

/// Bezier curve interpolate/extrapolate and tangent calculation with the elements given.
/// This mutates the elements, such copying them first is necessary!
/// Panics if not at least 1 elements exist.
fn bezier_with_tangent<R,P,T>(mut elements: P, scalar: R) -> [T;2]
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Sub<Output = T> + Copy,
    R: Real + FromPrimitive
{
    let len = elements.as_mut().len();
    if len < 2 {
        // if we have less than two elements, we just return the one element and a zero out vector.
        return [elements.as_mut()[0], elements.as_mut()[0] * R::zero()];
    }
    triangle_folding_inline(elements.as_mut(), |first, second| first * (R::one()-scalar) + second * scalar , len - 2);
    let elements = elements.as_mut();
    let tangent = (elements[1] - elements[0]) * R::from_usize(len-1).unwrap();
    let result = elements[0]*(R::one()-scalar)+elements[1]*scalar;
    [result, tangent]
}

/// Bezier curve interpolation and deriative calculation with the elements given.
/// This returns an array [v,d1,d2,..] with v interpolated value, d1 as the first deriative and so on.
/// This mutates the elements, such copying them first is necessary!
/// Panics if no element exists.
fn bezier_with_deriatives<R,P,T,const K: usize>(mut elements: P, scalar: R) -> [T;K]
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<R, Output = T> + Sub<Output = T> + Copy,
    R: Real + FromPrimitive
{
    let len = elements.as_mut().len();
    let deg = K.min(len-1);
    triangle_folding_inline(elements.as_mut(), |first, second| first * (R::one()-scalar) + second * scalar, len - deg -1);
    // take a zero out vector which can be copied to initialise the array (and have the right default)
    let mut grad = [elements.as_mut()[0] * R::zero();K];
    for k in (1..=deg).rev() {
        //calculate difference folding
        let grad_slice = &mut grad[..=k];
        lower_triangle_folding_inline(grad_slice, |first, second| second - first, k);
        let prod = R::from_usize((len-k..len).product::<usize>()).unwrap();
        grad[k] = grad[k] * prod;
        // do one step of the normal folding
        triangle_folding_inline(elements.as_mut(), |first, second| first * (R::one()-scalar) + second * scalar, 1);
        // copy the necessary data over to grad
        grad[..k].clone_from_slice(&elements.as_mut()[..k]);
    }
    grad
}

/// Bezier curve with given elements.
#[derive(Debug, Copy, Clone)]
pub struct Bezier<R,E,S>
{
    elements: E,
    space: S,
    _input: PhantomData<*const R>,
}

impl Bezier<Unknown, Unknown, Unknown> {
    /// Get a builder for bezier curves.
    ///
    /// The builder takes:
    /// - elements with `elements` or `elements_with_weights`
    /// - the type of input with `input`
    /// - the kind of workspace to use with `dynamic`, `constant` or `workspace`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::error::Error;
    /// # use enterpolation::{bezier::Bezier, Generator, Curve};
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// #
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let bez = Bezier::builder()
    ///     .elements([20.0,100.0,0.0,200.0])?
    ///     .input::<f64>()
    ///     .constant()
    ///     .build();
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
    pub fn builder() -> BezierBuilder<Unknown, Unknown, Unknown, Unknown>{
        BezierBuilder::new()
    }
}

impl<R,E,S> Bezier<R,E,S>
where
    E: DiscreteGenerator,
    S: Space<E::Output>,
{
    /// Creates a workspace and copies all elements into it.
    fn workspace(&self) -> impl AsMut<[E::Output]>{
        let mut workspace = self.space.workspace();
        let mut_workspace = workspace.as_mut();
        for i in 0..self.elements.len(){
            mut_workspace[i] = self.elements.gen(i);
        }
        workspace
    }
}

impl<R,E,S> Generator<R> for Bezier<R,E,S>
where
    E: DiscreteGenerator,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy,
    S: Space<E::Output>,
    R: Real
{
    type Output = E::Output;
    fn gen(&self, scalar: R) -> E::Output {
        // we pass only slices to guarantee the size of workspace to match the number of elements
        bezier(&mut self.workspace().as_mut()[..self.elements.len()], scalar)
    }
}

impl<R,E,S> Interpolation<R> for Bezier<R,E,S>
where
    E: DiscreteGenerator,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy,
    S: Space<E::Output>,
    R: Real
{}

impl<R,E,S> Curve<R> for Bezier<R,E,S>
where
    E: DiscreteGenerator,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy,
    S: Space<E::Output>,
    R: Real
{
    /// Return the domain of the Curve, in this case just [0.0,1.0].
    fn domain(&self) -> [R; 2] {
        [R::zero(),R::one()]
    }
}

impl<R,E,S> Bezier<R,E,S>
where
    E: DiscreteGenerator,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Sub<Output = E::Output> + Copy,
    S: Space<E::Output>,
    R: Real + FromPrimitive
{
    /// Generate the value and its tangent, in this order.
    pub fn gen_with_tangent(&self, scalar: R) -> [E::Output;2] {
        // we pass only slices to guarantee the size of workspace to match the number of elements
        bezier_with_tangent(&mut self.workspace().as_mut()[..self.elements.len()], scalar)
    }

    /// Generate the value and its deriatives, the order hereby is from value, then firt deriative, then second and so on.
    pub fn gen_with_deriatives<const K: usize>(&self, scalar: R) -> [E::Output;K] {
        // we pass only slices to guarantee the size of workspace to match the number of elements
        bezier_with_deriatives(&mut self.workspace().as_mut()[..self.elements.len()], scalar)
    }
}

impl<R,E,S> Bezier<R,E,S>
where
    E: DiscreteGenerator,
    S: Space<E::Output>,
{
    /// Create generic bezier curve.
    ///
    /// Building a bezier curve with the associated builder is recommended.
    pub fn new(elements: E, space: S) -> Result<Self,BezierError> {
        if space.len() < elements.len() {
            return Err(TooSmallWorkspace::new(space.len(),elements.len()).into());
        }
        if elements.is_empty(){
            return Err(Empty::new().into());
        }
        Ok(Bezier {
            space,
            elements,
            _input: PhantomData,
        })
    }

    /// Create generic bezier curve without doing any checking.
    ///
    /// Building a bezier curve with the associated builder is recommended.
    ///
    /// # Panics
    ///
    /// May panic or return non-expected values if the space given is less than the number of elements.
    /// Will panic if the given generator does not generate any element.
    pub fn new_unchecked(elements: E, space: S) -> Self {
        Bezier {
            space,
            elements,
            _input: PhantomData,
        }
    }
}

impl<R,E,S> Bezier<R,E,S>
where
    E: DiscreteGenerator + Extend<<E as Generator<usize>>::Output> + IndexMut<usize, Output = <E as Generator<usize>>::Output>,
    <E as Generator<usize>>::Output: Add<Output = <E as Generator<usize>>::Output> + Mul<R, Output = <E as Generator<usize>>::Output> + Copy,
    R: Real + FromPrimitive
{
    /// Elevates the curve such that it's degree increases by one.
    pub fn elevate_inplace(&mut self){
        //TODO: test if workspace is big enough, otherwise return error?!
        //TODO: otherwise increase workspace -> DynSpace would by a trait, current DynSpace would be renamed to VecSpace
        //TODO: this trait would have something like "set_len" which would set the length of the space created.
        //TODO: this would allow our space to work with the newly grown elements.
        //TODO: however we would like to be able to give a bigger workspace in the beginning (workspace) and elevating just fills in the space...
        //TODO: so we only want to test? And one has to find out the space needed at creation?
        let stepper = Stepper::normalized(self.elements.len() + 2);
        self.elements.extend(core::iter::once(self.elements.last().unwrap()));
        //TODO: instead of last and temp we could just reverse our order (go from n to 1)
        let mut last = self.elements.gen(0);
        for (i, factor) in stepper.enumerate().skip(1).take(self.elements.len()) {
            let temp = self.elements.gen(i);
            self.elements[i] = last * factor + self.elements.gen(i) * (R::one()-factor);
            last = temp;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ConstSpace;

    #[test]
    fn extrapolation() {
        let bez = Bezier::builder()
            .elements([20.0,0.0,200.0]).unwrap()
            .input::<f64>()
            .constant()
            .build();
        assert_f64_near!(bez.gen(2.0), 820.0);
        assert_f64_near!(bez.gen(-1.0), 280.0);
    }

    #[test]
    fn bigger_workspace() {
        let bez = Bezier::new([5.0], ConstSpace::<_,3>::new()).unwrap();
        let res = bez.gen_with_tangent(0.5);
        assert_f64_near!(res[0], 5.0);
        assert_f64_near!(res[1], 0.0);
    }

    #[test]
    fn constant() {
        let bez = Bezier::builder()
            .elements([5.0]).unwrap()
            .input::<f64>()
            .constant()
            .build();
        let res = bez.gen_with_tangent(0.25);
        assert_f64_near!(res[0], 5.0);
        assert_f64_near!(res[1], 0.0);
        let res = bez.gen_with_tangent(0.75);
        assert_f64_near!(res[0], 5.0);
        assert_f64_near!(res[1], 0.0);
    }

    #[test]
    fn deriatives(){
        let bez = Bezier::builder()
            .elements([1.0,2.0,3.0]).unwrap()
            .input::<f64>()
            .constant()
            .build();
        let res = bez.gen_with_tangent(0.5);
        assert_f64_near!(res[0], 2.0);
        assert_f64_near!(res[1], 2.0);
        let res = bez.gen_with_deriatives::<3>(0.5);
        assert_f64_near!(res[0], 2.0);
        assert_f64_near!(res[1], 2.0);
        assert_f64_near!(res[2], 0.0);
        // check if asking of to many deriatives does not panic
        let res = bez.gen_with_deriatives::<5>(0.5);
        assert_f64_near!(res[0], 2.0);
        assert_f64_near!(res[1], 2.0);
        assert_f64_near!(res[2], 0.0);
        assert_f64_near!(res[3], 0.0);
        assert_f64_near!(res[4], 0.0);
    }

}
