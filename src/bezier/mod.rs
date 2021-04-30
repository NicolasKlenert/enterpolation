// TODO: creation of Interpolations should not panic, instead it should return a Result!
// TODO: rational Bezier curves!

pub mod bezier_deriative;

use core::ops::{Add, Mul, Sub};
use core::marker::PhantomData;
use crate::{Interpolation, Stepper};
use crate::utils::{triangle_folding_inline, lower_triangle_folding_inline};

// TODO: this function could in theory take a collection of references, create it's own container
// TODO: and generate directly in the container. This could have better performance.
// TODO: However for this to work, we would have to create a collection space with a length of 1 less
// TODO: then the given collection. Such we would have to differentiate between
// TODO: static and dynamic size OR create a trait in which this is handled for us
// TODO: -> the trait would have something like toOwned for creation (a specific with_capacity function)
// TODO: which would return a collection of MaybeUnInit. This would be unsafe, however improve (possibly) performace
/// Bezier curve interpolate/extrapolate with the elements given.
/// This mutates the elements, such copying them first is necessary!
/// Panics if not at least 1 element exists.
pub fn bezier<P,T>(mut elements: P, scalar: f64) -> T
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let len = elements.as_mut().len();
    triangle_folding_inline(elements.as_mut(), |first, second| first * (1.0-scalar) + second * scalar , len);
    elements.as_mut()[0]
}

// TODO: instead of bezier_with_tangent have a bezier_with_deriatives
// TODO: which would take a number (const usize or usize) and return an array with [get, first deriative, second ...]
// TODO: this can be easily done as first we do our normale get algo and then after n-k steps we would copy the collection
// TODO: from here on out we would calculate the deriative (to the last place of the collection)
// TODO: then we would do a step of the get algo and then copy all elements to the collection (the last place is safe
// TODO: as we have 1 element less). This goes on until we ware finished -> a lot of for loops but otherwise safe!
/// Bezier curve interpolate/extrapolate and tangent calculation with the elements given.
/// This mutates the elements, such copying them first is necessary!
/// Panics if not at least 2 elements exist.
pub fn bezier_with_tangent<P,T>(mut elements: P, scalar: f64) -> [T;2]
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Sub<Output = T> + Copy
{
    let len = elements.as_mut().len();
    triangle_folding_inline(elements.as_mut(), |first, second| first * (1.0-scalar) + second * scalar , len - 1);
    let elements = elements.as_mut();
    let tangent = (elements[1] - elements[0]) * len as f64;
    let result = elements[0]*(1.0-scalar)+elements[1]*scalar;
    [result, tangent]
}

/// Bezier curve interpolation and deriative calculation with the elements given.
/// This returns an array [v,d1,d2,..] with v interpolated value, d1 as the first deriative and so on.
/// This mutates the elements, such copying them first is necessary!
/// Panics if not at least K+1 elements exist.
pub fn bezier_with_deriatives<P,T,const K: usize>(mut elements: P, scalar: f64) -> [T;K]
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Sub<Output = T> + Copy
{
    let len = elements.as_mut().len();
    triangle_folding_inline(elements.as_mut(), |first, second| first * (1.0-scalar) + second * scalar, len - K);
    // take the first element which can be copied to initialise the array
    let mut grad = [elements.as_mut()[0];K];
    for k in (1..=K).rev() {
        //calculate difference folding
        let grad_slice = &mut grad[..k];
        lower_triangle_folding_inline(grad_slice, |first, second| second - first, k);
        //the multiplication with f64 loses precision, there could be a better way
        let prod : f64 = (len-k..len).product::<usize>() as f64;
        grad[k] = grad[k] * prod;
        // do one step of the normal folding
        triangle_folding_inline(elements.as_mut(), |first, second| first * (1.0-scalar) + second * scalar, 1);
        // copy the necessary data over to grad
        for i in 0..k {
            grad[i] = elements.as_mut()[i];
        }
    }
    grad
}

/// Trims the given bezier curve at the point given by scalar.
/// Mutates the given elements such that they represent the right half of the bezier curve.
pub fn bezier_trim_left<P,T>(mut elements: P, scalar: f64)
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let len = elements.as_mut().len();
    lower_triangle_folding_inline(elements.as_mut(), |first, second| first * (1.0-scalar) + second * scalar, len);
}

/// Trims the given bezier curve at the point given by scalar.
/// Mutates the given elements such that they represent the left half of the bezier curve.
pub fn bezier_trim_right<P,T>(mut elements: P, scalar: f64)
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let len = elements.as_mut().len();
    triangle_folding_inline(elements.as_mut(), |first, second| first * (1.0-scalar) + second * scalar, len);
}

/// Elevates the curve such that it's degree increases by one.
pub fn bezier_elevate_inplace<T>(elements: &mut Vec<T>)
where T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let stepper = Stepper::new(elements.len() + 2);
    elements.push(*elements.last().unwrap());
    //TODO: instead of last and temp we could just reverse our order (go from n to 1)
    let mut last = elements[0];
    for (i, factor) in stepper.enumerate().skip(1).take(elements.len()) {
        let temp = elements[i];
        elements[i] = last * factor + elements[i] * (1.0-factor);
        last = temp;
    }
}

/// Elevates the curve and outputs it. The given buffer is used.
/// The buffer needs to have a length of 1 bigger then the current one.
/// That is, it must have a length of degree + 2
pub fn bezier_elevate<P,Q,T>(source: P, mut target: Q)
where
    P: AsRef<[T]>,
    Q: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    let other = target.as_mut();
    let me = source.as_ref();
    assert!(other.len() == me.len() + 1);
    let stepper = Stepper::new(me.len() + 2);
    other[0] = me[0];
    other[me.len()] = me[me.len()-1];
    for (i, factor) in stepper.enumerate().skip(1).take(me.len()) {
        other[i] = me[i-1] * factor + me[i] * (1.0-factor);
    }
}


pub struct Bezier<P,T>
{
    elements: P,
    element: PhantomData<T>
}

impl<P,T> Interpolation for Bezier<P,T>
where
    P: AsRef<[T]> + ToOwned,
    P::Owned: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    type Input = f64;
    type Output = T;
    fn get(&self, scalar: f64) -> T {
        bezier(self.elements.to_owned(), scalar)
    }
}

impl<P,T> Bezier<P,T>
where
    P: AsRef<[T]> + ToOwned,
    P::Owned: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Sub<Output = T> + Copy
{
    pub fn get_with_tangent(&self, scalar: f64) -> [T;2] {
        bezier_with_tangent(self.elements.to_owned(), scalar)
    }

    pub fn get_with_deriatives<const K: usize>(&self, scalar: f64) -> [T;K] {
        bezier_with_deriatives(self.elements.to_owned(), scalar)
    }
}

impl<T> Bezier<Vec<T>,T>
{
    /// Creates a bezier curve of elements given.
    /// There has to be at least 2 elements.
    pub fn from_collection<C>(collection: C) -> Self
    where C: IntoIterator<Item = T>
    {
        let elements: Vec<T> = collection.into_iter().collect();
        assert!(elements.len() > 1);
        Bezier {
            elements,
            element: PhantomData
        }
    }
}

impl<T> Bezier<Vec<T>,T>
where T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    /// Elevates the curve such that it's degree increases by one.
    pub fn elevate_inplace(&mut self){
        bezier_elevate_inplace(&mut self.elements)
    }
}

impl<P,T> Bezier<P,T>
where P: AsRef<[T]>
{
    /// Creates a bezier curve of elements given.
    /// There has to be at least 2 elements.
    pub fn new(elements: P) -> Self
    {
        assert!(elements.as_ref().len() > 1);
        Bezier {
            elements,
            element: PhantomData
        }
    }
}

impl<P,T> Bezier<P,T>
where
    P: AsRef<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    /// Elevates the curve and outputs it. The given buffer is used.
    /// The buffer needs to have a length of 1 bigger then the current one.
    /// That is, it must have a length of degree + 2
    pub fn elevate<Q>(&self, mut elements: Q) -> Bezier<Q,T>
    where Q: AsRef<[T]> + AsMut<[T]>
    {
        bezier_elevate(self.elements.as_ref(),elements.as_mut());
        Bezier::new(elements)
    }
}

impl<P,T> Bezier<P,T>
where
    P: AsMut<[T]>,
    T: Add<Output = T> + Mul<f64, Output = T> + Copy
{
    pub fn trim_left(&mut self, scalar: f64){
        bezier_trim_left(self.elements.as_mut(), scalar)
    }

    pub fn trim_right(&mut self, scalar: f64){
        bezier_trim_right(self.elements.as_mut(), scalar)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Interpolation, Curve};

    #[test]
    fn linear() {
        let bez = Bezier::new([20.0,100.0,0.0,200.0]);
        let mut iter = bez.take(5);
        let expected = [20.0,53.75,65.0,98.75,200.0];
        for i in 0..=4 {
            let val = iter.next().unwrap();
            assert_f64_near!(val, expected[i]);
        }
    }

    #[test]
    fn extrapolation() {
        let bez = Bezier::new([20.0,0.0,200.0]);
        assert_f64_near!(bez.get(2.0), 820.0);
        assert_f64_near!(bez.get(-1.0), 280.0);
    }

    #[test]
    fn constant() {
        let bez = Bezier::new([5.0,5.0]);
        let res = bez.get_with_tangent(0.25);
        assert_f64_near!(res[0], 5.0);
        assert_f64_near!(res[1], 0.0);
        let res = bez.get_with_tangent(0.75);
        assert_f64_near!(res[0], 5.0);
        assert_f64_near!(res[1], 0.0);
    }

}
