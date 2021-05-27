//! Linear Interpolations
//! These Interpolations can be stacked together to create multidimensional Interpolations
//! Linear Interplations are one of the simplest forms of interpolations.
//! Most of the time, Linear Interpolations are used as an approximation of curves, such
//! Linear Interpolations often do have many elements. For this reason
//! we supply the specialized LinearEquidistant Interplation, in which we assume that the distance
//! of an element to it's neighbors is constant. This increases performance, as the search for
//! the border elements to calculate the linear interpolation with can be found in O(1)
//! instead of O(log n) with n being the number of elements in the interpolation structure.

// TODO: creation of Interpolations should not panic, instead it should return a Result!
use core::ops::{Add, Mul};
use core::marker::PhantomData;
use crate::{Generator, Interpolation, Curve, EnterpolationError, SortedList,
    FiniteGenerator, Equidistant, ConstEquidistant};
use crate::real::Real;
use crate::utils::upper_border;
use num_traits::cast::FromPrimitive;

use core::fmt::Debug;

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
fn linear<R,T,K,E>(elements: &E, knots: &K, scalar: R) -> T
where
    E: Generator<usize, Output = T>,
    K: SortedList<R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Debug
{
    //we use upper_border_with_factor as this allows us a performance improvement for equidistant knots
    let (min_index, max_index, factor) = knots.upper_border_with_factor(scalar);
    let min_point = elements.get(min_index);
    let max_point = elements.get(max_index);
    min_point * (R::one() - factor) + max_point * factor
}

/// Linear Interpolation Structure with knots
/// If knots are roughly or exactly equidistant, consider using LinearEquidistant instead.
pub struct Linear<R,T,K,E>
{
    elements: E,
    knots: K,
    _phantoms: (PhantomData<R>, PhantomData<T>)
}

impl<R,T,K,E> Generator<R> for Linear<R,T,K,E>
where
    E: Generator<usize, Output = T>,
    K: SortedList<R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Debug
{
    type Output = T;
    fn get(&self, scalar: R) -> T {
        linear(&self.elements, &self.knots, scalar)
    }
}

impl<R,T,K,E> Interpolation<R> for Linear<R,T,K,E>
where
    E: Generator<usize, Output = T>,
    K: SortedList<R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Debug
{}

impl<R,T,K,E> Curve<R> for Linear<R,T,K,E>
where
    E: Generator<usize, Output = T>,
    K: SortedList<R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Debug
{
    fn domain(&self) -> [R; 2] {
        [self.knots.first().unwrap(), self.knots.last().unwrap()]
    }
}

impl<R,T> Linear<R,T,Vec<R>,Vec<T>>
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
        if elements.len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 2,
            });
        }
        let knots: Vec<R> = elements.iter().enumerate().map(func).collect();
        Ok(Linear {
            elements,
            knots,
            _phantoms: (PhantomData, PhantomData)
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
            _phantoms: (PhantomData, PhantomData)
        })
    }
}

impl<R,T> Linear<R,T,Equidistant<R>,Vec<T>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
    /// Creates a linear interpolation of elements given with equidistant knots.
    /// There has to be at least 2 elements.
    pub fn from_collection<C>(collection: C) -> Result<Self, EnterpolationError>
    where C: IntoIterator<Item = T>
    {
        let elements: Vec<T> = collection.into_iter().collect();
        if elements.len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 2,
            });
        }
        Ok(Linear {
            knots: Equidistant::new(elements.len()),
            elements,
            _phantoms: (PhantomData, PhantomData)
        })
    }
}

impl<R,T,K,E> Linear<R,T,K,E>
where
    E: FiniteGenerator<Output = T>,
    K: SortedList<R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real
{
    /// Create a linear interpolation with slice-like collections of elements and knots.
    /// Knots should be in increasing order (not checked), there should be as many knots as elements
    /// and there has to be at least 2 elements.
    pub fn new(elements: E, knots: K) -> Result<Self, EnterpolationError>
    {
        if elements.len() < 2 {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 2,
            });
        }
        if knots.len() != elements.len() {
            return Err(EnterpolationError::InvalidNumberKnots{
                name: "Linear".to_string(),
                found: knots.len(),
                expected: "same amount as elements".to_string(),
            });
        }
        Ok(Linear {
            elements,
            knots,
            _phantoms: (PhantomData, PhantomData)
        })
    }
}

impl<R,T,E> Linear<R,T,Equidistant<R>,E>
where
    E: FiniteGenerator<Output = T>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive
{
    /// Create a linear interpolation with slice-like collections of elements.
    /// There has to be at least 1 element.
    /// We assume the knots to be equidistant distributed.
    pub fn new_equidistant(elements: impl Into<E>) -> Result<Self, EnterpolationError>
    {
        let elements = elements.into();
        if elements.is_empty() {
            return Err(EnterpolationError::ToFewElements{
                name: "Linear".to_string(),
                found: elements.len(),
                expected: 1,
            });
        }
        Ok(Linear {
            knots: Equidistant::new(elements.len()),
            elements,
            _phantoms: (PhantomData, PhantomData)
        })
    }
}

impl<R,T,const N: usize> Linear<R,T,ConstEquidistant<R>,[T;N]>
{
    /// Create a linear interpolation with an array of elements.
    /// There has to be at least 1 element, which is NOT checked.
    /// Should be used if one wants to create a constant Interpolation
    pub const fn new_equidistant_unchecked(elements: [T;N]) -> Self
    {
        Linear {
            elements,
            knots: ConstEquidistant::new(N),
            _phantoms: (PhantomData, PhantomData)
        }
    }
}

/// An array-allocated, const-creatable, linear interpolation.
///
/// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
pub type ConstLinear<R,T,const N: usize> = Linear<R,T,ConstEquidistant<R>,[T;N]>;
/// An array-allocated linear interpolation.
///
/// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
pub type StaticLinear<R,T,const N: usize> = Linear<R,T,[R;N],[T;N]>;
/// A vector-allocated linear interpolation.
///
/// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
pub type DynamicLinear<R,T> = Linear<R,T,Vec<R>,Vec<T>>;
/// An array-allocated linear interpolation with equidistant knot distribution.
///
/// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
pub type StaticEquidistantLinear<R,T,const N: usize> = Linear<R,T,Equidistant<R>,[T;N]>;
/// A vector-allocated linear interpolation with equidistant knot distribution.
///
/// **Because this is an alias, not all its methods are listed here. See the [`Linear`](crate::linear::Linear) type too.**
pub type DynamicEquidistantLinear<R,T> = Linear<R,T,Equidistant<R>,Vec<T>>;


#[cfg(test)]
mod test {
    use super::*;
    use crate::Curve;

    #[test]
    fn linear_equidistant() {
        let lin = DynamicEquidistantLinear::<f64,_>::from_collection(vec![20.0,100.0,0.0,200.0]).unwrap();
        // let lin = Linear::<f64,_,_,_>::from_collection(vec![20.0,100.0,0.0,200.0]).unwrap();
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = lin.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn linear() {
        let lin = DynamicLinear::from_collection_with_knots(vec![(20.0,0.0),(100.0,1.0/3.0),(0.0,2.0/3.0),(200.0,1.0)]).unwrap();
        // let lin = Linear::<f64,_,_,_>::from_collection_with_knots(vec![(20.0,0.0),(100.0,1.0/3.0),(0.0,2.0/3.0),(200.0,1.0)]).unwrap();
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        let mut iter = lin.take(expected.len());
        for i in 0..expected.len() {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn extrapolation() {
        let lin = Linear::from_collection_with_knots(vec![(20.0,1.0),(100.0,2.0),(0.0,3.0),(200.0,4.0)]).unwrap();
        assert_f64_near!(lin.get(1.5), 60.0);
        assert_f64_near!(lin.get(2.5), 50.0);
        assert_f64_near!(lin.get(-1.0), -140.0);
        assert_f64_near!(lin.get(5.0), 400.0);
    }

    //TODO: try to get rid of turbofish... how?
    #[test]
    fn constant_equidistant(){
        let constant = StaticEquidistantLinear::new_equidistant([5.0]).unwrap();
        // let constant = Linear::<f64,_,Equidistant<f64>,CollectionWrapper<[f64;1],f64>>::new([5.0],Equidistant::new(1)).unwrap();
        assert_f64_near!(constant.get(-1.0), 5.0);
        assert_f64_near!(constant.get(10.0), 5.0);
        assert_f64_near!(constant.get(0.5), 5.0);
    }

    //TODO: add constant test (not equidistant)

    #[test]
    fn const_creation(){
        const LIN : ConstLinear<f64,f64,4> = ConstLinear::new_equidistant_unchecked([20.0,100.0,0.0,200.0]);
        // const LIN : Linear<f64,f64,ConstEquidistant<f64>,CollectionWrapper<[f64;4],f64>> = Linear::new_equidistant_unchecked([20.0,100.0,0.0,200.0]);
        let mut iter = LIN.take(7);
        let expected = [20.0,60.0,100.0,50.0,0.0,100.0,200.0];
        for i in 0..=6 {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }
}
