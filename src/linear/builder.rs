//! Builder module for linear interpolations.
//!
//! Each interpolation has it's own builder module, which accumalates all methods to create their interpolation.

//TODO: EXAMPLE

use core::ops::{Add, Mul};
use core::marker::PhantomData;
use num_traits::real::Real;
use num_traits::FromPrimitive;
use crate::{DiscreteGenerator, SortedList, Equidistant, Homogeneous, Weighted, NonEmptyGenerator};
use super::Linear;

// API:
// push_element
// push_element_with_weight
// push_knot
// extend (with all different iterators?->different names)

/// Struct indicator to mark if we use weights
#[derive(Debug)]
struct WithWeight<T>(T);

#[derive(Debug)]
struct Unknown;

#[derive(Debug)]
struct Output<R = f64>(PhantomData<*const R>);

impl<R> Output<R> {
    pub fn new() -> Self {
        Output(PhantomData)
    }
}

// /// Easy of use function to generate the builder. -> function inside of the struct itself (Linear::builder())
// pub fn builder<R,E>(elements: E) -> LinearBuilder<Output<R>,E> {
//     LinearBuilder::new(elements)
// }

#[derive(Debug, Clone)]
pub struct LinearBuilder<K,E> {
    knots: K,
    elements: E,
}

impl Default for LinearBuilder<Unknown, Unknown> {
    fn default() -> Self {
        LinearBuilder::new()
    }
}

impl LinearBuilder<Unknown, Unknown> {
    pub fn new() -> Self {
        LinearBuilder {
            knots: Unknown,
            elements: Unknown,
        }
    }
}

impl LinearBuilder<Unknown, Unknown> {
    pub fn elements<E>(elements: E) -> LinearBuilder<Unknown, E> {
        LinearBuilder {
            knots: Unknown,
            elements,
        }
    }
    //TODO: add from_collection?
}

impl<T,R> LinearBuilder<Unknown, WithWeight<Vec<Homogeneous<T,R>>>> {
    pub fn elements_with_weights<K,E>(elements: E, weights: K) -> Self
    where
        E: IntoIterator<Item = T>,
        K: IntoIterator<Item = R>,
        R: Copy,
        T: Mul<R, Output = T>,
    {
        let vec = elements.into_iter().zip(weights.into_iter())
            .map(|(element,weight)| Homogeneous::weighted(element,weight)).collect();
        LinearBuilder {
            knots: Unknown,
            elements: WithWeight(vec),
        }
    }
}

impl<T,R,const N: usize> LinearBuilder<Unknown, WithWeight<[Homogeneous<T,R>;N]>> {
    pub fn elements_with_weights(elements: [T;N], weights: [R;N]) -> Self
    where
        R: Default + Copy,
        T: Default + Copy + Mul<R, Output = T>,
    {
        let mut arr = [Default::default();N];
        for i in 0..N {
            arr[i] = Homogeneous::weighted(elements[i],weights[i]);
        }
        LinearBuilder {
            knots: Unknown,
            elements: WithWeight(arr),
        }
    }
}

impl<K,E> LinearBuilder<K,E>
where
    E: NonEmptyGenerator,
    K: SortedList,
    E::Output: Add<Output = E::Output> + Mul<K::Output, Output = E::Output> + Copy,
    K::Output: Real
{
    pub fn build(self) -> Linear<K,E>{
        Linear::new(self.elements, self.knots).unwrap()
    }
}

impl<K,T,R> LinearBuilder<K,WithWeight<Vec<Homogeneous<T,R>>>>
where
    K: SortedList<Output = R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Copy
{
    pub fn build(self) -> Weighted<Linear<K,Vec<Homogeneous<T,R>>>,T,R>{
        Weighted::new(Linear::new(self.elements.0, self.knots).unwrap())
    }
}

impl<K,T,R, const N: usize> LinearBuilder<K,WithWeight<[Homogeneous<T,R>;N]>>
where
    K: SortedList<Output = R>,
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Copy
{
    pub fn build(self) -> Weighted<Linear<K,[Homogeneous<T,R>;N]>,T,R>{
        Weighted::new(Linear::new(self.elements.0, self.knots).unwrap())
    }
}

impl<E> LinearBuilder<Unknown, E>
{
    pub fn knots<K>(self, knots: K) -> LinearBuilder<K,E>
    where
        K: SortedList
    {
        LinearBuilder {
            knots,
            elements: self.elements,
        }
    }

    pub fn equidistant<R>(self) -> LinearBuilder<Output<R>,E>{
        LinearBuilder {
            knots: Output::new(),
            elements: self.elements,
        }
    }
}

impl<R,E> LinearBuilder<Output<R>, E>
where
    E: NonEmptyGenerator,
    E::Output: Add<Output = E::Output> + Mul<R, Output = E::Output> + Copy,
    R: Real + FromPrimitive
{
    pub fn build(self) -> Linear<Equidistant<R>,E> {
        let len = self.elements.len();
        Linear::new(self.elements, Equidistant::normalized(len)).unwrap()
    }

    pub fn build_with_domain(self, start:R, end: R) -> Linear<Equidistant<R>,E> {
        let len = self.elements.len();
        Linear::new(self.elements, Equidistant::new(start, end, len)).unwrap()
    }
}

impl<T,R> LinearBuilder<Output<R>,WithWeight<Vec<Homogeneous<T,R>>>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + Copy + FromPrimitive
{
    /// Build a weighted linear interpolation from a vector of elements and equidistant knots in [0.0,1.0].
    pub fn build(self) -> Weighted<Linear<Equidistant<R>,Vec<Homogeneous<T,R>>>,T,R>{
        let len = self.elements.0.len();
        Weighted::new(Linear::new(self.elements.0, Equidistant::normalized(len)).unwrap())
    }
    /// Build a weighted linear interpolation from a vector of elements and equidistant knots in the specified domain.
    pub fn build_with_domain(self, start:R, end: R) -> Weighted<Linear<Equidistant<R>,Vec<Homogeneous<T,R>>>,T,R> {
        let len = self.elements.0.len();
        Weighted::new(Linear::new(self.elements.0, Equidistant::new(start, end, len)).unwrap())
    }
}

impl<T,R, const N: usize> LinearBuilder<Output<R>,WithWeight<[Homogeneous<T,R>;N]>>
where
    T: Add<Output = T> + Mul<R, Output = T> + Copy,
    R: Real + FromPrimitive + Copy
{
    /// Build a weighted linear interpolation from an array of elements and equidistant knots in [0.0,1.0].
    pub fn build(self) -> Weighted<Linear<Equidistant<R>,[Homogeneous<T,R>;N]>,T,R>{
        let len = self.elements.0.len();
        Weighted::new(Linear::new(self.elements.0, Equidistant::normalized(len)).unwrap())
    }
    /// Build a weighted linear interpolation from an array of elements and equidistant knots in the specified domain.
    pub fn build_with_domain(self, start:R, end: R) -> Weighted<Linear<Equidistant<R>,[Homogeneous<T,R>;N]>,T,R> {
        let len = self.elements.0.len();
        Weighted::new(Linear::new(self.elements.0, Equidistant::new(start, end, len)).unwrap())
    }
}
