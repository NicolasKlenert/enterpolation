//! Linear Interpolations.
//!
//! Linear Interplations are one of the simplest forms of interpolations.
//! Most of the time, Linear Interpolations are used as an approximation of curves, such
//! Linear Interpolations often do have many elements. For this reason
//! we supply the specialized LinearEquidistant Interplation, in which we assume that the distance
//! of an element to it's neighbors is constant. This increases performance, as the search for
//! the border elements to calculate the linear interpolation with can be found in O(1)
//! instead of O(log n) with n being the number of elements in the interpolation structure.

//TODO: do not use MinSize -> too much noise and we still have to check if #elements = #knots
//TODO: instead use only Sorted and test for the other two (minsize 2 and same number) in the new function
//TODO: only have new and new_unchecked in this module. All other constructions should be done in the builder!

use core::ops::{Add, Mul};
use crate::{Generator, Interpolation, Curve, EnterpolationError, Sorted, SortedGenerator,
    DiscreteGenerator, Equidistant, ConstEquidistant, Homogeneous};
use crate::utils::upper_border;
use num_traits::real::Real;
use num_traits::cast::FromPrimitive;

use core::fmt::Debug;

// mod hyper;
mod builder;
pub use builder::LinearBuilder;

pub mod error;
pub use error::{LinearError, ToFewElements, KnotElementInequality, WeightOfZero, NotSorted};

/// Linear interpolate/extrapolate with the elements and knots given.
/// Knots should be in increasing order and there has to be at least 2 knots.
/// Also there has to be the same amount of elements and knots.
/// These constrains are not checked!
pub fn linear_array<R,T,K,E>(elements: E, knots: K, scalar: R) -> T
where
    E: AsRef<[T]>,
    K: AsRef<[R]>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{
    let (min_index, max_index) = upper_border(knots.as_ref(), scalar);
    let min = knots.as_ref()[min_index];
    let max = knots.as_ref()[max_index];
    let min_point = elements.as_ref()[min_index];
    let max_point = elements.as_ref()[max_index];
    let factor = (scalar - min) / (max - min);
    min_point * (R::one() - factor) + max_point * factor
}

/// Linear interpolate/extrapolate with the elements and knots given.
/// Knots should be in increasing order and there has to be at least 2 knots.
/// Also there has to be the same amount of elements and knots.
/// These constrains are not checked!
fn linear<R,K,E>(elements: &E, knots: &K, scalar: R) -> E::Output
where
    E: DiscreteGenerator,
    K: SortedGenerator<Output = R>,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy + Debug,
    R: Real + Debug
{
    //we use upper_border_with_factor as this allows us a performance improvement for equidistant knots
    let (min_index, max_index, factor) = knots.upper_border(scalar);
    let min_point = elements.gen(min_index);
    let max_point = elements.gen(max_index);
    min_point * (R::one() - factor) + max_point * factor
}

/// Linear Interpolation Structure with knots
/// If knots are roughly or exactly equidistant, consider using LinearEquidistant instead.
///
/// K has to be a SortedGenerator and and also a generator with a minimum size of 2.
/// Also this struct itself checks if the number of knots and elements are equal.
pub struct Linear<K,E>
{
    elements: E,
    knots: K,
}

impl Linear<builder::Unknown,builder::Unknown> {
    /// Get the builder object of this interpolation.
    pub fn builder() -> LinearBuilder<builder::Unknown,builder::Unknown> {
        LinearBuilder::new()
    }
}

impl<R,K,E> Generator<R> for Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator<Output = R>,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy + Debug,
    R: Real + Debug
{
    type Output = E::Output;
    /// # Panics
    ///
    /// Panics if `scalar` is NaN or similar.
    fn gen(&self, scalar: K::Output) -> Self::Output {
        linear(&self.elements, &self.knots, scalar)
    }
}

impl<R,K,E> Interpolation<R> for Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator<Output = R>,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy + Debug,
    R: Real + Debug
{}

impl<R,K,E> Curve<R> for Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator<Output = R>,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy + Debug,
    R: Real + Debug
{
    fn domain(&self) -> [R; 2] {
        [self.knots.first().unwrap(), self.knots.last().unwrap()]
    }
}

impl<R,T> Linear<Sorted<Vec<R>>,Vec<T>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
    /// Create a linear interpolation with at least 2 elements.
    /// Knots are calculated with the given closure, which takes the index and the reference to the element.
    /// Knots should be in increasing order. This is not checked.
    /// For a constant speed of the curve, the distance between the elements should be used.
    pub fn from_collection_with<C,F>(collection: C, func: F) -> Result<Self, EnterpolationError>
    where
        C: IntoIterator<Item = T>,
        F: FnMut((usize,&T)) -> R,
    {
        let elements: Vec<T> = collection.into_iter().collect();
        // if elements.len() < 2 {
        //     return Err(EnterpolationError::ToFewElements{
        //         name: "Linear".to_string(),
        //         found: elements.len(),
        //         expected: 2,
        //     });
        // }
        let knots: Vec<R> = elements.iter().enumerate().map(func).collect();
        Ok(Linear {
            elements,
            knots: Sorted::new(knots).unwrap(),
        })
    }

    /// Create a linear interpolation of the elements with given knots.
    /// Knots should be in increasing order and there has to be at least 2 elements.
    /// The increasing order of knots is not checked.
    pub fn from_collection_with_knots<C>(collection: C) -> Result<Self, EnterpolationError>
    where C: IntoIterator<Item = (T, R)>
    {
        let iter = collection.into_iter();
        let mut elements: Vec<T> = Vec::with_capacity(iter.size_hint().0);
        let mut knots: Vec<R> = Vec::with_capacity(iter.size_hint().0);
        for (elem, knot) in iter {
            elements.push(elem);
            knots.push(knot);
        }
        // if elements.len() < 2 {
        //     return Err(EnterpolationError::ToFewElements{
        //         name: "Linear".to_string(),
        //         found: elements.len(),
        //         expected: 2,
        //     });
        // }
        Ok(Linear {
            elements,
            knots: Sorted::new(knots).unwrap(),
        })
    }
}

impl<R,T> Linear<Vec<R>,Vec<Homogeneous<T,R>>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
    // /// Create a linear interpolation with at least 2 elements.
    // /// Knots are calculated with the given closure, which takes the index and the reference to the element.
    // /// Knots should be in increasing order. This is not checked.
    // /// For a constant speed of the curve, the distance between the elements should be used.
    // pub fn from_collection_with<C,F>(collection: C, func: F) -> Result<Self, EnterpolationError>
    // where
    //     C: IntoIterator<Item = T>,
    //     F: FnMut((usize,&T)) -> R,
    // {
    //     let elements: Vec<T> = collection.into_iter().collect();
    //     if elements.len() < 2 {
    //         return Err(EnterpolationError::ToFewElements{
    //             name: "Linear".to_string(),
    //             found: elements.len(),
    //             expected: 2,
    //         });
    //     }
    //     let knots: Vec<R> = elements.iter().enumerate().map(func).collect();
    //     Ok(Linear {
    //         elements,
    //         knots,
    //     })
    // }

    /// Create a linear interpolation of the elements with given knots and weights.
    /// Knots should be in increasing order and there has to be at least 2 elements.
    /// The increasing order of knots is not checked.
    pub fn from_collection_with_weights_and_knots<C>(collection: C) -> Result<Self, EnterpolationError>
    where C: IntoIterator<Item = (T, R, R)>
    {
        let iter = collection.into_iter();
        let mut elements: Vec<Homogeneous<T,R>> = Vec::with_capacity(iter.size_hint().0);
        let mut knots: Vec<R> = Vec::with_capacity(iter.size_hint().0);
        for (elem, weight, knot) in iter {
            elements.push(Homogeneous::weighted(elem, weight).unwrap());
            knots.push(knot);
        }
        if elements.len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 2,
            });
        }
        Ok(Linear {
            elements,
            knots,
        })
    }
}

impl<R,T> Linear<Equidistant<R>,Vec<T>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
    /// Creates a linear interpolation of elements given with equidistant knots inside [0.0,1.0].
    /// There has to be at least 2 elements.
    pub fn from_collection<C>(collection: C) -> Result<Self, EnterpolationError>
    where C: IntoIterator<Item = T>
    {
        let elements: Vec<T> = collection.into_iter().collect();
        // if elements.len() < 2 {
        //     return Err(EnterpolationError::ToFewElements{
        //         name: "Linear".to_string(),
        //         found: elements.len(),
        //         expected: 2,
        //     });
        // }
        Ok(Linear {
            knots: Equidistant::normalized(elements.len()),
            elements,
        })
    }
}

impl<K,E> Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    E::Output: Add<Output = E::Output> + Mul<K::Output, Output = E::Output> + Copy,
    K::Output: Real
{
    /// Create a linear interpolation with slice-like collections of elements and knots.
    /// Knots should be in increasing order (not checked), there should be as many knots as elements
    /// and there has to be at least 2 elements.
    pub fn new(elements: E, knots: K) -> Result<Self, LinearError>
    {
        if elements.len() < 2 {
            return Err(ToFewElements::new(elements.len()).into());
        }
        if knots.len() != elements.len() {
            return Err(KnotElementInequality::new(elements.len(), knots.len()).into());
        }
        Ok(Linear {
            elements,
            knots,
        })
    }
}

impl<K,E> Linear<K,E>
where
    E: DiscreteGenerator,
    K: SortedGenerator,
    E::Output: Add<Output = E::Output> + Mul<K::Output, Output = E::Output> + Copy,
    K::Output: Real
{
    /// Create a linear interpolation with slice-like collections of elements and knots.
    /// Knots should be in increasing order, there should be as many knots as elements
    /// and there has to be at least 2 elements.
    ///
    /// All requirements are not checked.
    pub fn new_unchecked(elements: E, knots: K) -> Self
    {
        Linear {
            elements,
            knots,
        }
    }
}

impl<R,T,const N: usize> Linear<ConstEquidistant<R,N>,[T;N]>
{
    /// Create a linear interpolation with an array of elements.
    ///
    /// There has to be at least *two* elements, which is NOT checked.
    /// This function should be used if one wants to create a constant Interpolation.
    ///
    /// # Requirements
    ///
    /// The array has to be at least of length *two*.
    pub const fn equidistant_unchecked(elements: [T;N]) -> Self
    {
        Linear {
            elements,
            knots: ConstEquidistant::new(),
        }
    }
}

// /// An array-allocated linear interpolation.
// ///
// /// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
// pub type StaticLinear<R,T,const N: usize> = Linear<Sorted<[R;N]>,[T;N]>;
// /// A vector-allocated linear interpolation.
// ///
// /// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
// pub type DynamicLinear<R,T> = Linear<Sorted<Vec<R>>,Vec<T>>;
// /// An array-allocated linear interpolation with equidistant knot distribution.
// ///
// /// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
// pub type StaticEquidistantLinear<R,T,const N: usize> = Linear<Equidistant<R>,[T;N]>;
// /// A vector-allocated linear interpolation with equidistant knot distribution.
// ///
// /// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
// pub type DynamicEquidistantLinear<R,T> = Linear<Equidistant<R>,Vec<T>>;
/// An array-allocated, const-creatable, linear interpolation with equidistant knot distribution.
///
/// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
pub type ConstEquidistantLinear<R,T,const N: usize> = Linear<ConstEquidistant<R,N>,[T;N]>;


#[cfg(test)]
mod test {
    use super::*;
    use crate::Curve;

    #[test]
    fn linear_equidistant() {
        //DynamicEquidistantLinear
        let lin = Linear::builder()
            .elements(vec![20.0,100.0,0.0,200.0]).unwrap()
            .equidistant::<f64>()
            .build();
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = lin.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn linear() {
        //DynamicLinear
        let lin = Linear::builder()
            .elements(vec![20.0,100.0,0.0,200.0]).unwrap()
            .knots(vec![0.0,1.0/3.0,2.0/3.0,1.0]).unwrap()
            .build();
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = lin.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn extrapolation() {
        let lin = Linear::builder()
            .elements([20.0,100.0,0.0,200.0]).unwrap()
            .knots([1.0,2.0,3.0,4.0]).unwrap()
            .build();
        assert_f64_near!(lin.gen(1.5), 60.0);
        assert_f64_near!(lin.gen(2.5), 50.0);
        assert_f64_near!(lin.gen(-1.0), -140.0);
        assert_f64_near!(lin.gen(5.0), 400.0);
    }

    #[test]
    fn weights(){
        let lin = Linear::builder()
            .elements_with_weights(vec![(0.0,9.0),(1.0,1.0)]).unwrap()
            .equidistant::<f64>()
            .build();
        assert_f64_near!(lin.gen(0.5), 0.1);
        // const LIN : Linear<f64,f64,ConstEquidistant<f64>,CollectionWrapper<[f64;4],f64>> = Linear::new_equidistant_unchecked([20.0,100.0,0.0,200.0]);
    }

    #[test]
    fn const_creation(){
        const LIN : ConstEquidistantLinear<f64,f64,4> = ConstEquidistantLinear::equidistant_unchecked([20.0,100.0,0.0,200.0]);
        // const LIN : Linear<f64,f64,ConstEquidistant<f64>,CollectionWrapper<[f64;4],f64>> = Linear::new_equidistant_unchecked([20.0,100.0,0.0,200.0]);
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = LIN.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }
}
