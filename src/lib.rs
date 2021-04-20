#[macro_use]
extern crate assert_float_eq;

pub mod linear;
pub mod bezier;
pub mod utils;

use thiserror::Error;

// Scalar which represents an inbetween two points (usually between 0.0 and 1.0) (from and to constants?!)
type InterScalar = f64;

/// Trait for all 1-dim Interpolations, which gets mutated when asked for an interpolation (to make them more efficient)
pub trait MutInterpolation {
    type Output;
    fn get(&mut self, scalar: InterScalar) -> Self::Output;
}

/// Trait for all 1-dim Interpolations
pub trait Interpolation {
    type Output;
    fn get(&self, scalar: InterScalar) -> Self::Output;
    fn extract<I>(&self, iterator: I) -> Extractor<Self, I>
    where I: Iterator<Item = InterScalar>
    {
        Extractor {
            interpolation: self,
            iterator
        }
    }
    fn take(&self, samples: usize) -> Extractor<Self, Stepper> {
        self.extract(Stepper::new(samples))
    }
}

#[derive(Error, Debug)]
pub enum EnterpolationError {
    #[error("To few elements given for creation of `{name}`, {found} elements given, but at least {expected} are necessary.")]
    ToFewElements{
        name: String,
        found: usize,
        expected: usize
    },
    #[error("The amount of knots given for creation of `{name}` are not correct, {found} knots given, but {expected} necessary.")]
    InvalidNumberKnots{
        name: String,
        found: usize,
        expected: String
    },
}

/// Iterator adaptor, which transforms an iterator with InterScalar items to an iterator with the correspondending values of the interpolation
pub struct Extractor<'a, T: ?Sized,I> {
    interpolation: &'a T,
    iterator: I
}

impl<'a, T, I> Iterator for Extractor<'a, T, I>
where
    T: Interpolation,
    I: Iterator<Item = InterScalar>
{
    type Item = T::Output;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.interpolation.get(self.iterator.next()?))
    }
}

/// Iterator which steps from 0.0 to 1.0 in a specific amount of constant steps.
pub struct Stepper {
    current: usize,
    amount: usize,
    inverse_amount: InterScalar,
}

impl Stepper {
    pub fn new(steps: usize) -> Self {
        Stepper {
            current: 0,
            amount: steps - 1,
            inverse_amount: 1.0 / (steps - 1) as InterScalar
        }
    }
}

impl Iterator for Stepper {
    type Item = InterScalar;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.amount {
            return None;
        }
        let res = self.current as InterScalar * self.inverse_amount;
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
