use core::ops::{Add, Mul};
use core::marker::PhantomData;
use num_traits::real::Real;
use num_traits::FromPrimitive;
use crate::{DiscreteGenerator, SortedList, Equidistant};
use super::Linear;

// Refactor: Instead of trying to accomodate all different collection types (with K and E),
// we go the easy route and just look at arrays and vecs.
// If someone wants another collection, the new method on the interpolation itself can still be used!

#[derive(Debug)]
pub struct Unknown<R = f64>(PhantomData<*const R>);

impl<R> Unknown<R> {
    pub fn new() -> Self {
        Unknown(PhantomData)
    }
}

/// Easy of use function to generate the builder.
pub fn builder<R,E>(elements: E) -> LinearBuilder<Unknown<R>,E> {
    LinearBuilder::new(elements)
}

//we may use a default of creating a LinearBuilder with a Vec...
#[derive(Debug, Clone)]
pub struct LinearBuilder<K,E> {
    knots: K,
    elements: E,
}

impl<R,E> LinearBuilder<Unknown<R>, E> {
    pub fn new(elements: E) -> Self {
        LinearBuilder {
            knots: Unknown::new(),
            elements,
        }
    }
}

// add from collection with iterator (element), (element,knot), (element,weight) and (element,weight,knot)?

// impl<R,T> LinearBuilder<Vec<R>, Vec<T>> {
//     pub fn from_collection(col: I) -> Self
//     where I: IntoIterator
//     {
//         LinearBuilder {
//             knots: Unknown::new(),
//             elements,
//         }
//     }
// }

impl<K,E> LinearBuilder<K,E>
where
    E: DiscreteGenerator,
    K: SortedList,
    E::Output: Add<Output = E::Output> + Mul<K::Output, Output = E::Output> + Copy,
    K::Output: Real
{
    pub fn build(self) -> Linear<K,E>{
        Linear::new(self.elements, self.knots).unwrap()
    }
}

impl<R,E> LinearBuilder<Unknown<R>, E>
{
    pub fn knots<K>(self, knots: K) -> LinearBuilder<K,E>
    where
        K: SortedList<Output = R>
    {
        LinearBuilder {
            knots,
            elements: self.elements,
        }
    }
}

impl<R,E> LinearBuilder<Unknown<R>, E>
where
    E: DiscreteGenerator,
    R: Real + FromPrimitive
{
    pub fn set_domain(self, start: R, end: R) -> LinearBuilder<Equidistant<R>,E>{
        LinearBuilder {
            knots: Equidistant::new(start, end, self.elements.len()),
            elements: self.elements,
        }
    }
}

impl<R,E> LinearBuilder<Unknown<R>, E>
where
    E: DiscreteGenerator,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy,
    R: Real + FromPrimitive
{
    pub fn build(self) -> Linear<Equidistant<R>,E> {
        let len = self.elements.len();
        Linear::new(self.elements, Equidistant::normalized(len)).unwrap()
    }
}
