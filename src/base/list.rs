use core::cmp::Ordering;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Div, Index, Sub};
use num_traits::FromPrimitive;
use num_traits::identities::Zero;
use num_traits::real::Real;

#[cfg(feature = "std")]
use std::error::Error;

//temp
use core::fmt::Debug;

use super::{Chain, Signal};

// REMARK: It may be valuable to create traits SortedNonEmpty and SortedNonSingular
// REMARK: These would be Sorted + NonEmpty and Sorted + MinSize<2>.
// REMARK: They would implement the specified functions without risk of panics and possible use of the functions.
// REMARK: However this will create even more traits which have to be implemented.

/// Trait to mark a chain as sorted.
///
/// This guarantees that the generated elements of a chain are
/// - comparable (you could define the trait Ord for the set of all generated elements)
/// - non-strictly increasing
///
/// Also it implements default search functions which can be overridden to achieve better performance
/// and accuracy.
///
/// # Panics
///
/// Some or all of this functions *may* panic. Each function has it's own panic segment which
/// describes its needs. To guarantee no panics at runtime, one should use struct or traits which
/// guarantee the needs of the functions. The MinSize trait was created just for this.
///
/// # Implementation
///
/// If a default implementation of a function is overwritten, the documentation should be copied
/// and the examples only slightly changed such that they are working. The values and equations
/// in the examples given should always be true. If some values in the examples can't be reproduced,
/// the example doesn't have to be copied. These cases (if applicable) should be tested:
/// - stricly increasing array with values
///     - outside of the array (both sides)
///     - first knot
///     - last knot
///     - a knot inside the array
///     - value inbetween two knots
/// - semi-constant array with values outside of the array (both_sides)
pub trait SortedChain: Chain {
    /// Returns the smallest index between `min` and `max`
    /// for which the corresponding element is bigger then the input.
    /// If all elements are smaller, this function will return the given maximum.
    ///
    /// #Panic
    ///
    /// Panics if `min` or `max` are not within [0,self.len()].
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, Sorted};
    /// let arr = Sorted::new_unchecked([0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0]);
    /// assert_eq!(arr.strict_upper_bound_clamped(-1.0,1,5),1);
    /// assert_eq!(arr.strict_upper_bound_clamped(0.15,1,5),2);
    /// assert_eq!(arr.strict_upper_bound_clamped(0.7,1,5),5);
    /// assert_eq!(arr.strict_upper_bound_clamped(20.0,1,5),5);
    /// ```
    fn strict_upper_bound_clamped(&self, element: Self::Output, min: usize, max: usize) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        let mut pointer = min;
        let mut dist = max - min;
        while dist > 0 {
            let step = dist / 2;
            let sample = pointer + step;
            if element >= self.eval(sample) {
                pointer = sample + 1;
                dist -= step + 1;
            } else {
                dist = step;
            }
        }
        pointer
    }
    /// Returns the smallest index for which the corresponding element is bigger then the input.
    /// If all elements are bigger, this function will return self.len().
    ///
    /// #Panic
    ///
    /// Panics if `self` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, Sorted};
    /// let arr = Sorted::new_unchecked([0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0]);
    /// assert_eq!(arr.strict_upper_bound(-1.0),0);
    /// assert_eq!(arr.strict_upper_bound(0.15),2);
    /// assert_eq!(arr.strict_upper_bound(0.7),6);
    /// assert_eq!(arr.strict_upper_bound(20.0),8);
    /// ```
    fn strict_upper_bound(&self, element: Self::Output) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        self.strict_upper_bound_clamped(element, 0, self.len())
    }

    /// Find the values inside the collection for which the given element is inbetween
    /// and a linear factor at how close it is to which value.
    ///
    /// This function in general returns indices corresponding to values (`first` and `second`)
    /// such that `first <= value <= second` is true.
    ///
    /// If the given element is smaller/bigger than every element in the collection, then
    /// the indices given will be the smallest/biggest possible and if the corresponding values are the same
    /// the factor is only guaranteed to be a valid number.
    ///
    /// # Remark
    ///
    /// There are collections for which the returned values of this function are not uniquely defined.
    /// You may not assume any other invariant except
    /// `first * factor + second * (1.0 - factor) == value`,
    /// *if* `first <= value <= second` holds true,
    /// where `value` is the value inserted into this function,
    /// and the function returned `(index_of_first, index_of_second, factor)`.
    ///
    /// Otherwise it may return any valid factor such that
    /// `first * factor + second * (1.0 - factor) == first == second`
    /// holds true.
    ///
    /// # Panics
    ///
    /// Panics if `self` is has less than *two* elements.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, Sorted, Signal};
    /// # use enterpolation::utils;
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// let arr = Sorted::new_unchecked([0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0]);
    /// let values = vec![-1.0,0.0,0.15,0.7,1.0,20.0];
    /// for value in values {
    ///     let (min_index, max_index, factor) = arr.upper_border(value);
    ///     let min = arr.eval(min_index);
    ///     let max = arr.eval(max_index);
    ///     assert_f64_near!(utils::lerp(min,max,factor),value);
    /// }
    /// ```
    ///
    /// ```
    /// # use enterpolation::{SortedChain, Sorted, Signal};
    /// # use enterpolation::utils;
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// let arr = Sorted::new_unchecked([0.0,0.0,5.0,5.0,5.0]);
    /// let values = vec![-1.0,20.0];
    /// let results = vec![0.0,5.0];
    /// for (value, result) in values.into_iter().zip(results) {
    ///     let (min_index, max_index, factor) = arr.upper_border(value);
    ///     println!("min_index: {:?}, max_index: {:?}, factor: {:?}", min_index, max_index, factor);
    ///     let min = arr.eval(min_index);
    ///     let max = arr.eval(max_index);
    ///     assert_f64_near!(utils::lerp(min,max,factor),result);
    /// }
    /// ```
    fn upper_border(&self, element: Self::Output) -> (usize, usize, Self::Output)
    where
        Self::Output: PartialOrd
            + Sub<Output = Self::Output>
            + Div<Output = Self::Output>
            + Zero
            + Copy
            + Debug,
    {
        let max_index = self.strict_upper_bound(element);
        // test if we have to clamp max_index -> if so, factor has to be calculated with a check for NaN.
        if self.len() == max_index {
            let max_index = self.len() - 1;
            let min_index = max_index - 1;
            return (
                min_index,
                max_index,
                self.linear_factor(min_index, max_index, element),
            );
        }
        if max_index == 0 {
            let max_index = 1;
            let min_index = 0;
            return (
                min_index,
                max_index,
                self.linear_factor(min_index, max_index, element),
            );
        }
        (
            max_index - 1,
            max_index,
            self.linear_factor_unchecked(max_index - 1, max_index, element),
        )
    }

    /// Calculate the factor of `element` inbetween `min` and `max`.
    ///
    /// That is, the factor would be needed to generate `element` from a linear interpolation of
    /// `min` and `max`, with `min` being the element generated by `min_index` and the same holds for `max_index`.
    ///
    /// This function may try to divide by zero if both elements behind the indices are the same.
    /// This is not checked.
    fn linear_factor_unchecked(
        &self,
        min_index: usize,
        max_index: usize,
        element: Self::Output,
    ) -> Self::Output
    where
        Self::Output: Sub<Output = Self::Output> + Div<Output = Self::Output> + Copy,
    {
        let max = self.eval(max_index);
        let min = self.eval(min_index);
        (element - min) / (max - min)
    }

    /// Calculate the factor of `element` inbetween `min` and `max`.
    ///
    /// That is, the factor would be needed to generate `element` from a linear interpolation of
    /// `min` and `max`, with `min` being the element generated by `min_index` and the same holds for `max_index`.
    ///
    /// If the factor could be anything, as both elements are the same, 1.0 is returned.
    fn linear_factor(
        &self,
        min_index: usize,
        max_index: usize,
        element: Self::Output,
    ) -> Self::Output
    where
        Self::Output: Sub<Output = Self::Output> + Div<Output = Self::Output> + Zero + Copy,
    {
        let max = self.eval(max_index);
        let min = self.eval(min_index);
        let div = max - min;
        if div.is_zero() {
            return div;
        }
        (element - min) / div
    }
    // If you want to add a default implementation: The wrapper `Sorted` should forward to the implementation!
}

/// Struct to represent a sorted collection.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Sorted<C>(C);

impl<C> Sorted<C>
where
    C: Chain,
    C::Output: PartialOrd,
{
    /// Returns Some(Sorted) if collection is sorted, otherwise returns `NotSorted` Error.
    pub fn new(col: C) -> Result<Self, NotSorted> {
        if col.is_empty() {
            return Ok(Sorted(col));
        }
        let mut last = col.eval(0);
        for i in 1..col.len() {
            let current = col.eval(i);
            match last.partial_cmp(&current) {
                None | Some(Ordering::Greater) => return Err(NotSorted { index: i }),
                _ => {
                    last = current;
                }
            }
        }
        Ok(Sorted(col))
    }
}

impl<C> Sorted<C> {
    /// Creates a sorted collection without checking if it is sorted.
    ///
    /// As unsorted collection will not create UB but will probably panic at some point,
    /// such this function is still safe, even if an unsorted collection is given.
    pub const fn new_unchecked(col: C) -> Self {
        Sorted(col)
    }
}

impl<C> Signal<usize> for Sorted<C>
where
    C: Signal<usize>,
{
    type Output = C::Output;
    fn eval(&self, input: usize) -> Self::Output {
        self.0.eval(input)
    }
}

impl<C> Chain for Sorted<C>
where
    C: Chain,
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<C: Chain> SortedChain for Sorted<C> {}

impl<C, Idx> Index<Idx> for Sorted<C>
where
    C: Index<Idx>,
{
    type Output = C::Output;
    fn index(&self, index: Idx) -> &Self::Output {
        self.0.index(index)
    }
}

/// Error returned if the given knots are not sorted.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NotSorted {
    index: usize,
}

impl NotSorted {
    /// Create a new error in which from index to index + 1 the values were decreasing.
    pub fn new(index: usize) -> Self {
        NotSorted { index }
    }
}

impl fmt::Display for NotSorted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Given knots are not sorted. From index {} to {} we found decreasing values.",
            self.index,
            self.index + 1
        )
    }
}

#[cfg(feature = "std")]
impl Error for NotSorted {}

/// Struct used as a signal for equidistant elements.
/// Acts like an array of knots.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Equidistant<R = f64> {
    len: usize,
    step: R,
    offset: R,
}

// // implement separate new functions to be able to call them with const -> see issue #57563
// impl Equidistant<f64>
// {
//     pub const fn new_f64(len: usize) -> Self {
//         Equidistant {
//             len,
//             step: 1.0/((len - 1) as f64) //-> not possible, see issue #57241
//         }
//     }
// }

impl<R> Equidistant<R>
where
    R: Real + FromPrimitive,
{
    /// Create a signal for equidistant real numbers with `len-1` steps from 0.0 to 1.0.
    ///
    /// #Panics
    ///
    /// Panics if the given length is 0 or `length -  1` can not be transformed into R.
    pub fn normalized(len: usize) -> Self {
        Equidistant {
            len,
            step: R::from_usize(len - 1).unwrap().recip(),
            offset: R::zero(),
        }
    }

    /// Create a signal for equidistant real numbers with `len-1` steps from `start` to `end`.
    ///
    /// #Panics
    ///
    /// Panics if the given length is 0 or `length -  1` can not be transformed into R.
    pub fn new(len: usize, start: R, end: R) -> Self {
        Equidistant {
            len,
            step: (end - start) / R::from_usize(len - 1).unwrap(),
            offset: start,
        }
    }

    /// Create a signal for equidistant real number with `len-1` steps from `start` to `end`.
    pub fn step(len: usize, start: R, step: R) -> Self {
        Equidistant {
            len,
            step,
            offset: start,
        }
    }
}

impl<R> Signal<usize> for Equidistant<R>
where
    R: Real + FromPrimitive,
{
    type Output = R;
    fn eval(&self, input: usize) -> R {
        self.step * R::from_usize(input).unwrap() + self.offset
    }
}

impl<R> Chain for Equidistant<R>
where
    R: Real + FromPrimitive,
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<R> SortedChain for Equidistant<R>
where
    R: Real + FromPrimitive,
{
    /// Returns the smallest index for which the corresponding element is bigger then the input.
    /// If all elements are bigger, this function will return self.len().
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, Equidistant};
    /// let equi = Equidistant::normalized(11);
    /// assert_eq!(equi.strict_upper_bound(-1.0),0);
    /// assert_eq!(equi.strict_upper_bound(0.15),2);
    /// assert_eq!(equi.strict_upper_bound(20.0),11);
    /// ```
    fn strict_upper_bound(&self, element: Self::Output) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        // extrapolation to the left
        if element < self.offset {
            return 0;
        }
        let scaled = (element - self.offset) / self.step;
        // now unrwapping is fine as we are above zero.
        let min_index = scaled.floor().to_usize().unwrap();
        self.len().min(min_index + 1)
    }
    /// Returns the smallest index between `min` and `max`
    /// for which the corresponding element is bigger then the input.
    /// If all elements are bigger, this function will return the given maximum.
    ///
    /// #Panic
    ///
    /// Panics if `min` or `max` are not within [0,self.len()].
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, Equidistant};
    /// let equi = Equidistant::normalized(11);
    /// assert_eq!(equi.strict_upper_bound_clamped(-1.0,1,3),1);
    /// assert_eq!(equi.strict_upper_bound_clamped(0.15,1,3),2);
    /// assert_eq!(equi.strict_upper_bound_clamped(20.0,1,3),3);
    /// ```
    fn strict_upper_bound_clamped(&self, element: Self::Output, min: usize, max: usize) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        // extrapolation to the left
        if element < self.eval(min) {
            return min;
        }
        let scaled = (element - self.offset) / self.step;
        // now unrwapping is fine as we are above zero.
        let min_index = scaled.floor().to_usize().unwrap();
        max.min(min_index + 1)
    }
    /// Find the values inside the collection for which the given element is inbetween
    /// and a linear factor at how close it is to which value.
    ///
    /// This function in general returns indices corresponding to values (`first` and `second`)
    /// such that `first <= value <= second` is true.
    ///
    /// If the given element is smaller/bigger than every element in the collection, then
    /// the indices given will be the smallest/biggest possible and if the corresponding values are the same
    /// the factor is only guaranteed to be a valid number.
    ///
    /// # Remark
    ///
    /// There are collections for which the returned values of this function are not uniquely defined.
    /// You may not assume any other invariant except
    /// `first * factor + second * (1.0 - factor) == value`,
    /// *if* `first <= value <= second` holds true,
    /// where `value` is the value inserted into this function,
    /// and the function returned `(index_of_first, index_of_second, factor)`.
    ///
    /// Otherwise it may return any valid factor such that
    /// `first * factor + second * (1.0 - factor) == first == second`
    /// holds true.
    ///
    /// # Panics
    ///
    /// May Panic if `self` is has less than *two* elements.
    /// Also panics if length-1 as usize can not be converted to `R`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, Equidistant, Signal};
    /// # use enterpolation::utils;
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// let equdist = Equidistant::normalized(6);
    /// let values = vec![-1.0,0.0,0.15,0.6,1.0,20.0];
    /// for value in values {
    ///     let (min_index, max_index, factor) = equdist.upper_border(value);
    ///     let min = equdist.eval(min_index);
    ///     let max = equdist.eval(max_index);
    ///     assert_f64_near!(utils::lerp(min,max,factor),value);
    /// }
    /// ```
    fn upper_border(&self, element: R) -> (usize, usize, R) {
        let scaled = (element - self.offset) / self.step;
        // extrapolation to the left
        if element < self.offset {
            return (0, 1, scaled);
        }
        // now unrwapping is fine as we are above zero.
        let min_index = scaled.floor().to_usize().unwrap();
        let max_index = scaled.ceil().to_usize().unwrap();
        //extrapolation to the right
        if max_index >= self.len {
            return (
                self.len - 2,
                self.len - 1,
                scaled - R::from_usize(self.len - 2).unwrap(),
            );
        }
        let factor = scaled.fract();
        (min_index, max_index, factor)
    }
}

/// Struct used as a signal for equidistant elements in constant context.
/// Acts like an array of knots.
///
/// This struct is necessary as to date neither generic bounds nor floating point opterations are
/// allowed in constant functions. Such to be able to use Equidistant in a constant context,
/// we use this structure instead.
///
/// In comparison to `Equidistant`, this struct is slower (as it has to do more calculations) and
/// only represents knots in [0.0,1.0]. However as knot base for interpolations, it is more performant,
/// as we have the knowledge of the domain.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ConstEquidistant<R /* = f64*/, const N: usize>(PhantomData<*const R>);

impl<R, const N: usize> ConstEquidistant<R, N> {
    /// Create a list of equidistant real numbers.
    /// This struct should only be created in a constant context. Otherwise use Equidistant instead.
    pub const fn new() -> Self {
        ConstEquidistant(PhantomData)
    }
}

impl<R, const N: usize> Signal<usize> for ConstEquidistant<R, N>
where
    R: Real + FromPrimitive,
{
    type Output = R;
    fn eval(&self, input: usize) -> R {
        R::from_usize(input).unwrap() / R::from_usize(N - 1).unwrap()
    }
}

impl<R, const N: usize> Chain for ConstEquidistant<R, N>
where
    R: Real + FromPrimitive,
{
    fn len(&self) -> usize {
        N
    }
}

impl<R, const N: usize> SortedChain for ConstEquidistant<R, N>
where
    R: Real + FromPrimitive,
{
    /// Returns the smallest index for which the corresponding element is bigger then the input.
    /// If all elements are bigger, this function will return self.len().
    ///
    /// # Panics
    ///
    /// Panics if `N` is 0.
    /// May panic if `N-1` can not be converted to type `R`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, ConstEquidistant};
    /// let equi = ConstEquidistant::<f64,11>::new();
    /// assert_eq!(equi.strict_upper_bound(-1.0),0);
    /// assert_eq!(equi.strict_upper_bound(0.15),2);
    /// assert_eq!(equi.strict_upper_bound(20.0),11);
    /// ```
    fn strict_upper_bound(&self, element: Self::Output) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        // extrapolation to the left
        if element < R::zero() {
            return 0;
        }
        let scaled = element * R::from_usize(N - 1).unwrap();
        // now unrwapping is fine as we are above zero.
        let min_index = scaled.floor().to_usize().unwrap();
        self.len().min(min_index + 1)
    }
    /// Returns the smallest index between `min` and `max`
    /// for which the corresponding element is bigger then the input.
    /// If all elements are bigger, this function will return the given maximum.
    ///
    /// #Panic
    ///
    /// Panics if `min` or `max` are not within [0,self.len()].
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, ConstEquidistant};
    /// let equi = ConstEquidistant::<f64,11>::new();
    /// assert_eq!(equi.strict_upper_bound_clamped(-1.0,1,3),1);
    /// assert_eq!(equi.strict_upper_bound_clamped(0.15,1,3),2);
    /// assert_eq!(equi.strict_upper_bound_clamped(20.0,1,3),3);
    /// ```
    fn strict_upper_bound_clamped(&self, element: Self::Output, min: usize, max: usize) -> usize
    where
        Self::Output: PartialOrd + Copy,
    {
        // extrapolation to the left
        if element < self.eval(min) {
            return min;
        }
        let scaled = element * R::from_usize(N - 1).unwrap();
        // now unrwapping is fine as we are above zero.
        let min_index = scaled.floor().to_usize().unwrap();
        max.min(min_index + 1)
    }
    /// Find the values inside the collection for which the given element is inbetween
    /// and a linear factor at how close it is to which value.
    ///
    /// This function in general returns indices with values (`first` and `second`)
    /// such that `first <= value <= second` is true.
    ///
    /// If the given element is smaller/bigger than every element in the collection, then
    /// the indices given will be the smallest/biggest possible.
    ///
    /// # Remark
    ///
    /// There are collections for which the returned values of this function are not uniquely defined.
    /// You may not assume any other invariant except
    /// `first * factor + second * (1.0 - factor) == value`,
    /// where `value` is the value inserted into this function,
    /// and the function returned `(first, second, factor)`.
    ///
    /// # Panics
    ///
    /// Panics if `self` is has less than *two* elements.
    /// Also panics if length-1 as usize can not be converted to `R`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::{SortedChain, ConstEquidistant, Signal};
    /// # use enterpolation::utils;
    /// # use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
    /// let equdist = ConstEquidistant::<f64,6>::new();
    /// let values = vec![-1.0,0.0,0.15,0.6,1.0,20.0];
    /// for value in values {
    ///     let (min_index, max_index, factor) = equdist.upper_border(value);
    ///     let min = equdist.eval(min_index);
    ///     let max = equdist.eval(max_index);
    ///     assert_f64_near!(utils::lerp(min,max,factor),value);
    /// }
    /// ```
    fn upper_border(&self, element: R) -> (usize, usize, R)
    where
        R: PartialOrd + Sub<Output = R> + Div<Output = R> + Copy + Debug,
    {
        let scaled = element * R::from_usize(N - 1).unwrap();
        // extrapolation to the left
        if element < R::zero() {
            return (0, 1, scaled);
        }
        // now unrwapping is fine as we are above zero.
        let min_index = scaled.floor().to_usize().unwrap();
        let max_index = scaled.ceil().to_usize().unwrap();
        //extrapolation to the right
        if max_index >= N {
            return (N - 2, N - 1, scaled - R::from_usize(N - 2).unwrap());
        }
        let factor = scaled.fract();
        (min_index, max_index, factor)
    }
}
