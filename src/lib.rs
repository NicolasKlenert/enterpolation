#[macro_use]
extern crate assert_float_eq;

pub mod linear;

// Scalar which represents an inbetween two points (usually between 0.0 and 1.0) (from and to constants?!)
type InterScalar = f64;

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

/// Find the indices in which the given element is in between.
/// We assume that the collection is non-empty and ordered, to use binary search.
/// If one or more elements in the collections are exactly equal to the element,
/// the function will return a border in which either index returned
/// will be the index of an element equal to the element given.
/// If the given element is smaller/bigger than every element in the collection, then
/// the borders given will the the smallest/biggest possible
fn find_borders<C,T>(collection: C, element: T) -> (usize, usize)
where
    C: AsRef<[T]>,
    T: PartialOrd + Copy
{
    let mut min_index = 0;
    let mut max_index = collection.as_ref().len() - 1;

    while min_index < max_index - 1 {
        let index = min_index + (max_index - min_index) / 2;
        let sample = collection.as_ref()[index];

        if element < sample {
            max_index = index;
        } else {
            min_index = index;
        }
    }
    (min_index, max_index)
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

    #[test]
    fn find_borders() {
        let vec = vec![0.0,0.1,0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9,1.0];
        assert_eq!(super::find_borders(&vec,0.35),(3,4));
        assert_eq!(super::find_borders(&vec,-1.0),(0,1));
        assert_eq!(super::find_borders(&vec,20.0),(9,10));
        // test if element given es equal to a knot
        let borders = super::find_borders(&vec,0.5);
        assert!(borders.0 == 5 || borders.1 == 5);
    }

}
