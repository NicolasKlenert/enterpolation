//! This crate provides a myriad of different interpolation, extrapolation and animation methods.
//! Most notably it tries to be generic and modular. If instances of your type act somewhat like
//! a vector space, this crate will be able to interpolate, extrapolate and animate them.
//! TODO: describe more

#![warn(missing_docs)]

#[macro_use]
extern crate assert_float_eq;


pub mod linear;
pub mod bezier;
pub mod utils;
// pub mod point;
mod real;
mod never;

use core::marker::PhantomData;

use thiserror::Error;
use crate::real::Real;
use num_traits::cast::FromPrimitive;

//DEPRECATED: Scalar which represents an inbetween two points (usually between 0.0 and 1.0) (from and to constants?!)
type InterScalar = f64;

/// Trait for all 1-dim Interpolations, which gets mutated when asked for an interpolation (to make them more efficient)
pub trait MutInterpolation {
    type Output;
    fn get(&mut self, scalar: InterScalar) -> Self::Output;
}

/// Trait for all Interpolations
pub trait Interpolation
{
    /// The input type of the interpolation
    type Input;
    /// The output type of the interpolation
    type Output;
    /// Calculate the element at point `scalar`.
    fn get(&self, scalar: Self::Input) -> Self::Output;
    /// Helper function if one wants to sample the interpolation.
    /// It takes an iterator which yields items which are inputted into the `get` function
    /// and returns the output of the interpolation.
    fn extract<I>(&self, iterator: I) -> Extractor<Self, I>
    where I: Iterator<Item = Self::Input>
    {
        Extractor {
            interpolation: self,
            iterator,
        }
    }
}

//TODO: For now, because of the wrapper, we want to implement interpolations with
//TODO: impl Into<E> where E: ElementGenerator

pub trait ElementGenerator {
    type Input; //if no input is necessary, use never::Never
    type Output; //usually an array (AsMut<[T]>) over the elements T
    fn generate_elements(&self, input: Self::Input) -> Self::Output;
}

/// Wrapper for struct which implement AsRef<[T]>
/// such that we are able to implement the `ElementGenerator` trait for them.
/// In the future, one may be able to disregard this and implement the trait without this wrapper
struct ElementCollectionWrapper<P,T>
(
    P,
    PhantomData<T>,
);

impl<P,T> From<P> for ElementCollectionWrapper<P,T>
where P: AsRef<[T]>
{
    fn from(col: P) -> Self {
        ElementCollectionWrapper(col, PhantomData)
    }
}

impl<P,T> ElementGenerator for ElementCollectionWrapper<P,T>
where
    P: AsRef<[T]> + ToOwned,
    <P as ToOwned>::Owned: AsMut<[T]>,
{
    type Input = never::Never;
    type Output = <P as ToOwned>::Owned;
    fn generate_elements(&self, _input: Self::Input) -> Self::Output {
        self.0.to_owned()
    }
}

/// Newtype Take to encapsulate implementation details of the curve method take
pub struct Take<'a, C>(Extractor<'a, C, Stepper<C::Input>>)
where
    C: ?Sized + Curve,
    C::Input: Real;

impl<'a, C> Iterator for Take<'a, C>
where
    C: ?Sized + Curve,
    C::Input: Real + FromPrimitive,
{
    type Item = C::Output;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Curve is a specialized Interpolation which takes a real number as input
pub trait Curve : Interpolation
where Self::Input: Real
{
    fn take(&self, samples: usize) -> Extractor<Self, Stepper<Self::Input>>
    where Self::Input: FromPrimitive
    {
        self.extract(Stepper::new(samples))
    }
}

/// The error structure of this crate. Each possible error this crate could return is listed here.
#[derive(Error, Debug)]
pub enum EnterpolationError {
    /// Error returned if the elements given at the creation of an interpolation are to few.
    #[error("To few elements given for creation of `{name}`, {found} elements given, but at least {expected} are necessary.")]
    ToFewElements{
        /// The name of the Interpolation we wanted to create.
        name: String,
        /// The number of elements found.
        found: usize,
        /// The number of elements we need at least.
        expected: usize
    },
    /// Error if the number of knots are not correct at time of creation of an interpolation.
    #[error("The amount of knots given for creation of `{name}` are not correct, {found} knots given, but {expected} necessary.")]
    InvalidNumberKnots{
        /// The name of the Interpolation we wanted to create.
        name: String,
        /// The number of knots found.
        found: usize,
        /// Description how many knots are needed.
        expected: String
    },
}

/// Iterator adaptor, which transforms an iterator with InterScalar items to an iterator with the correspondending values of the interpolation
pub struct Extractor<'a, T: ?Sized, I> {
    interpolation: &'a T,
    iterator: I,
}

impl<'a, T, I> Iterator for Extractor<'a, T, I>
where
    T: ?Sized + Interpolation,
    I: Iterator<Item = T::Input>
{
    type Item = T::Output;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.interpolation.get(self.iterator.next()?))
    }
}

/// Iterator which steps from 0.0 to 1.0 in a specific amount of constant steps.
pub struct Stepper<R: Real = f64> {
    current: usize,
    amount: usize,
    inverse_amount: R,
}

impl<R> Stepper<R>
where
    R: Real + FromPrimitive,
{
    /// Creates a new Stepper stepping from 0 to 1
    /// The given generic real number has to be able to be created from usize
    /// Also the given steps are not allowed to be less than 1
    pub fn new(steps: usize) -> Self {
        Stepper {
            current: 0,
            amount: steps - 1,
            inverse_amount: R::from_usize(steps - 1).unwrap().recip()
        }
    }
}

impl<R> Iterator for Stepper<R>
where R: Real + FromPrimitive,
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.amount {
            return None;
        }
        let res = self.inverse_amount * R::from_usize(self.current).unwrap();
        self.current += 1;
        Some(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn stepper() {
        let mut stepper = Stepper::new(11);
        let res = vec![0.0,0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1.0];
        for i in 0..=10 {
            let val = stepper.next().unwrap();
            assert_f64_near!(val,res[i]);
        }
    }

}
