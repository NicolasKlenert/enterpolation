//! Module for different utilities which are used across other modules or to help the user of the library.
use num_traits::real::Real;
use core::ops::{Add,Mul};

/// Linear interpolation of the two values given.
pub fn lerp<T,R>(first: T, second: T, factor: R) -> T
where
    T: Add<Output = T> + Mul<R,Output = T>,
    R: Real
{
    first * (R::one()-factor) + second * factor
}

/// Calculate a pascalsche triangle with the given closure until the maximal steps as levels are reached.
/// If one wants to fold all values into the first position of the given buffer
/// a step size of the length of the buffer - 1 should be used.
pub fn triangle_folding_inline<P,T>(mut triangle: P, func: impl Fn(T,T) -> T, steps: usize)
where
    P: AsMut<[T]>,
    T: Copy
{
    let elements = triangle.as_mut();
    let len = elements.len();
    for k in 1..=steps {
        for i in 0..len-k {
            elements[i] = func(elements[i], elements[i+1]);
        }
    }
}

/// Calculate a pascalsche triangle with the given closure until the maximal steps as levels are reached.
/// If one wants to fold all values into the last position of the given buffer
/// a step size of the length of the buffer - 1 should be used.
pub fn lower_triangle_folding_inline<P,T>(mut triangle: P, func: impl Fn(T,T) -> T, steps: usize)
where
    P: AsMut<[T]>,
    T: Copy
{
    let elements = triangle.as_mut();
    let len = elements.len();
    for k in 1..=steps {
        for i in k..len {
            elements[i] = func(elements[i-1], elements[i]);
        }
    }
}

#[cfg(test)]
mod test {


}
