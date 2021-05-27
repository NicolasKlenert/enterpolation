use core::marker::PhantomData;
use core::ops::{Sub, Div};
use num_traits::real::Real;
use num_traits::FromPrimitive;

pub use super::{Generator, Interpolation, Curve, FiniteGenerator, Extract, Stepper};

/// Trait which symbolises a list of sorted elements, also outputs the value itself instead of a reference.
///
/// This trait is mostly used to achieve better performance and accuracy for interpolation with equidistant knots
/// without needing an extra struct.
pub trait SortedList<T> : FiniteGenerator<Output = T> {
    /// Find the biggest index to the corresponding element which is still smaller or equal to the element given.
    /// We assume that the collection is non-empty and ordered, to use binary search.
    /// If one or more elements in the collections are exactly equal to the element,
    /// the function will return the last duplicated element.
    /// If the given element is smaller/bigger than every element in the collection, then
    /// the function will return 0 or len()-1.
    /// As this function allows equality (such is not strict) it's counterpart upper_bound *can*
    /// return a smaller index than this function.
    ///
    /// # Panics
    ///
    /// Panics if `collection` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::utils::lower_bound;
    /// let arr = [0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0];
    /// assert_eq!(lower_bound(&arr,-1.0),0);
    /// assert_eq!(lower_bound(&arr,0.15),1);
    /// assert_eq!(lower_bound(&arr,0.7),5);
    /// assert_eq!(lower_bound(&arr,20.0),7);
    /// ```
    fn lower_bound(&self, element: T) -> usize
    where T: PartialOrd + Copy
    {
        if self.last().unwrap() <= element {
            return self.len() - 1;
        }
        self.upper_border(element)[0]
    }

    /// Find the smallest index to the corresponding element which is still bigger or equal to the element given.
    /// We assume that the collection is non-empty and ordered, to use binary search.
    /// If one or more elements in the collections are exactly equal to the element,
    /// the function will return the last duplicated element.
    /// If the given element is smaller/bigger than every element in the collection, then
    /// the function will return 0 or len()-1.
    /// As this function allows equality (such is not strict) it's counterpart lower_bound *can*
    /// return a bigger index than this function.
    ///
    /// # Panics
    ///
    /// Panics if `collection` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::utils::upper_bound;
    /// let arr = [0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0];
    /// assert_eq!(upper_bound(&arr,-1.0),0);
    /// assert_eq!(upper_bound(&arr,0.15),2);
    /// assert_eq!(upper_bound(&arr,0.7),3);
    /// assert_eq!(upper_bound(&arr,20.0),7);
    /// ```
    fn upper_bound(&self, element: T) -> usize
    where T: PartialOrd + Copy
    {
        if self.first().unwrap() >= element {
            return 0;
        }
        self.lower_border(element)[1]
    }

    /// This function has a slightly better performance in the specific case one only needs the max_index of the function
    /// upper_border. That is `strict_upper_bound(collection, element) == upper_border(collection, element).1`.
    /// Also they are diferent in the edge case that if all elements in the array are smaller, this function **will** return 0.
    /// `upper_border` on the other hand will return 1 (as the min_index occupies 0).
    /// If all elements are bigger, this function will return len()-1.
    ///
    /// #Panic
    ///
    /// Panics if the self is empty.
    ///
    fn strict_upper_bound(&self, element: T) -> usize
    where T: PartialOrd + Copy
    {
        let mut pointer = 0;
        let mut count = self.len();
        while count > 0 {
            let step = count / 2;
            let sample = pointer + step;
            if element >= self.get(sample){
                pointer = sample +1;
                count -= step +1;
            }else{
                count = step;
            }
        }
        pointer
    }

    /// This function has a slightly better performance in the specific case one only needs the min_index of the function
    /// lower_border. That is `strict_lower_bound(collection, element) == lower_border(collection, element).0`.
    /// Also they are diferent in the edge case that if all elements in the array are bigger, this function **will** return len() -1.
    /// `lower_border` on the other hand will return len()-2 (as the max_index occupies len()-1).
    /// If all elements are smaller, this function will return 0.
    ///
    /// #Panic
    ///
    /// Panics if the collection is empty.
    ///
    fn strict_lower_bound(&self, element: T) -> usize
    where T: PartialOrd + Copy
    {
        let mut pointer = self.len() - 1;
        let mut count = self.len();
        while count > 0 {
            let step = count / 2;
            let sample = pointer - step;
            if element >= self.get(sample){
                pointer = sample -1;
                count -= step +1;
            }else{
                count = step;
            }
        }
        pointer
    }

    /// Find the indices to the corresponding elements inside the collection
    /// for which the given element is inbetween.
    /// We assume that the collection is non-empty and ordered, to use binary search.
    /// If one or more elements in the collections are exactly equal to the element,
    /// the function will return a border where the smaller index correspondsto an element
    /// which is equal to the element given and the other index corresponds to a bigger element.
    /// If the given element is smaller/bigger than every element in the collection, then
    /// the borders given will be the smallest/biggest possible.
    ///
    /// # Panics
    ///
    /// Panics if `collection` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::utils::upper_border;
    /// let arr = [0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0];
    /// assert_eq!(upper_border(&arr,-1.0),(0,1));
    /// assert_eq!(upper_border(&arr,0.15),(1,2));
    /// assert_eq!(upper_border(&arr,0.7),(5,6));
    /// assert_eq!(upper_border(&arr,20.0),(6,7));
    /// ```
    fn upper_border(&self, element: T) -> [usize; 2]
    where
        T: PartialOrd + Copy
    {
        let mut min_index = 0;
        let mut max_index = self.len() - 1;

        while max_index - min_index > 1 {
            let index = min_index + (max_index - min_index) / 2;
            let sample = self.get(index);

            if element < sample {
                max_index = index;
            } else {
                min_index = index;
            }
        }
        [min_index, max_index]
    }

    /// Find the indices to the corresponding elements inside the collection
    /// for which the given element is inbetween and a factor at how much it is close by the elements.
    /// We assume that the collection is non-empty and ordered, to use binary search.
    /// If one or more elements in the collections are exactly equal to the element,
    /// the function will return a border where the smaller index correspondsto an element
    /// which is equal to the element given and the other index corresponds to a bigger element.
    /// If the given element is smaller/bigger than every element in the collection, then
    /// the borders given will be the smallest/biggest possible and the factor will be 0.0 or 1.0.
    ///
    /// This function is only there for a performance boost, as calculating the factor for the specific case
    /// of a border can be faster then the generic implementation here.
    ///
    /// # Panics
    ///
    /// Panics if `collection` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::utils::upper_border;
    /// let arr = [0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0];
    /// assert_eq!(upper_border(&arr,-1.0),(0,1));
    /// assert_eq!(upper_border(&arr,0.15),(1,2));
    /// assert_eq!(upper_border(&arr,0.7),(5,6));
    /// assert_eq!(upper_border(&arr,20.0),(6,7));
    /// ```
    fn upper_border_with_factor(&self, element: T) -> (usize, usize, T)
    where
        T: PartialOrd + Sub<Output = T> + Div<Output = T> + Copy
    {
        let [min_index, max_index] = self.upper_border(element);
        (min_index, max_index, self.factor(min_index, max_index, element))
    }

    /// Calculate the factor of `element` inbetween `min` and `max`.
    /// That is, the factor would be needed to generate `element` from a linear interpolation of
    /// `min` and `max`, with `min` being the element generated by `min_index` and the same holds for `max_index`.
    fn factor(&self, min_index: usize, max_index: usize, element: T) -> T
    where T: Sub<Output = T> + Div<Output = T> + Copy
    {
        let max = self.get(max_index);
        let min = self.get(min_index);
        (element - min) / (max - min)
    }

    /// Find the indices to the corresponding elements inside the collection
    /// for which the given element is inbetween.
    /// We assume that the collection is non-empty and ordered, to use binary search.
    /// If one or more elements in the collections are exactly equal to the element,
    /// the function will return a border where the bigger index correspondsto an element
    /// which is equal to the element given and the other index corresponds to a smaller element.
    /// If the given element is smaller/bigger than every element in the collection, then
    /// the borders given will be the smallest/biggest possible.
    ///
    /// # Panics
    ///
    /// Panics if `collection` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use enterpolation::utils::lower_border;
    /// let arr = [0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0];
    /// assert_eq!(lower_border(&arr,-1.0),(0,1));
    /// assert_eq!(lower_border(&arr,0.15),(1,2));
    /// assert_eq!(lower_border(&arr,0.7),(2,3));
    /// assert_eq!(lower_border(&arr,20.0),(6,7));
    /// ```
    fn lower_border(&self, element: T) -> [usize; 2]
    where
        T: PartialOrd + Copy
    {
        let mut min_index = 0;
        let mut max_index = self.len() - 1;

        while max_index - min_index > 1 {
            let index = min_index + (max_index - min_index) / 2;
            let sample = self.get(index);

            if element > sample {
                min_index = index;
            } else {
                max_index = index;
            }
        }
        [min_index, max_index]
    }
}

/// Struct used as a generator for equidistant elements.
/// Acts like an array of knots.
pub struct Equidistant<R>{
    len: usize,
    step: R,
}

// // implement seperate new functions to be able to call them with const -> see issue #57563
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
where R: Real + FromPrimitive
{
    /// Create a performant generator for equidistant real numbers with `len` steps.
    pub fn new(len: usize) -> Self {
        Equidistant {
            len,
            step: R::from_usize(len - 1).unwrap().recip()
        }
    }
}

impl<R> Generator<usize> for Equidistant<R>
where R: Real + FromPrimitive
{
    type Output = R;
    fn get(&self, input: usize) -> R {
        self.step * R::from_usize(input).unwrap()
    }
}

impl<R> FiniteGenerator for Equidistant<R>
where R: Real + FromPrimitive
{
    fn len(&self) -> usize {
        self.len
    }
}

//TODO: Returning an Option or such would be more idiomatic! -> what to do with 0 or 1 element?!
//TODO: upper_border is difficult to write for equidistant (making sure both indices are valid but are not the same!)
//TODO: we don't even have a contract for that, such we should think about it carefully!
//TODO: It is important to note that upper_border_with_factor does not act like upper_border -> Change the name!

impl<R> SortedList<R> for Equidistant<R>
where R: Real + FromPrimitive
{
    // /// # Panics
    // ///
    // /// Panics if the SortedList has less than 2 elements.
    // fn upper_border(&self, element: R) -> [usize; 2]
    // where R: PartialOrd + Copy
    // {
    //     let scaled = element * R::from_usize(self.len()-1).unwrap();
    //     let min_index = scaled.floor().to_usize().unwrap().min(self.len()-2).max(0);
    //     let max_index = scaled.ceil().to_usize().unwrap().min(self.len()-1).max(1);
    //     [min_index, max_index]
    // }

    fn upper_border_with_factor(&self, element: R) -> (usize, usize, R)
    where
        R: PartialOrd + Sub<Output = R> + Div<Output = R> + Copy
    {
        let scaled = element * R::from_usize(self.len()-1).unwrap();
        let min_index = scaled.floor().to_usize().unwrap().max(0);
        let max_index = scaled.ceil().to_usize().unwrap().min(self.len()-1);
        let factor = scaled.fract();
        (min_index, max_index, factor)
    }
}

/// Struct used as a generator for equidistant elements in constant context.
/// Acts like an array of knots.
///
/// This struct is necessary as to date neither generic bounds nor floating point opterations are
/// allowed in constant functions. Such to be able to use Equidistant in a constant context,
/// we use this structure instead.
pub struct ConstEquidistant<R>{
    len: usize,
    phantom: PhantomData<*const R>
}

impl<R> ConstEquidistant<R>
{
    /// Create a list of equidistant real numbers.
    /// This struct should only be created in a constant context. Otherwise use Equidistant instead.
    pub const fn new(len: usize) -> Self {
        ConstEquidistant {
            len,
            phantom: PhantomData
        }
    }
}

impl<R> Generator<usize> for ConstEquidistant<R>
where R: Real + FromPrimitive
{
    type Output = R;
    fn get(&self, input: usize) -> R {
        R::from_usize(self.len - 1).unwrap() / R::from_usize(input).unwrap()
    }
}

impl<R> FiniteGenerator for ConstEquidistant<R>
where R: Real + FromPrimitive
{
    fn len(&self) -> usize {
        self.len
    }
}

//TODO: Returning an Option or such would be more idiomatic! -> what to do with 0 or 1 element?!
//TODO: upper_border is difficult to write for equidistant (making sure both indices are valid but are not the same!)
//TODO: we don't even have a contract for that, such we should think about it carefully!
//TODO: It is important to note that upper_border_with_factor does not act like upper_border -> Change the name!

impl<R> SortedList<R> for ConstEquidistant<R>
where R: Real + FromPrimitive
{
    // /// # Panics
    // ///
    // /// Panics if the SortedList has less than 2 elements.
    // fn upper_border(&self, element: R) -> [usize; 2]
    // where R: PartialOrd + Copy
    // {
    //     let scaled = element * R::from_usize(self.len()-1).unwrap();
    //     let min_index = scaled.floor().to_usize().unwrap().min(self.len()-2).max(0);
    //     let max_index = scaled.ceil().to_usize().unwrap().min(self.len()-1).max(1);
    //     [min_index, max_index]
    // }

    fn upper_border_with_factor(&self, element: R) -> (usize, usize, R)
    where
        R: PartialOrd + Sub<Output = R> + Div<Output = R> + Copy
    {
        let scaled = element * R::from_usize(self.len()-1).unwrap();
        let min_index = scaled.floor().to_usize().unwrap().max(0);
        let max_index = scaled.ceil().to_usize().unwrap().min(self.len()-1);
        let factor = scaled.fract();
        (min_index, max_index, factor)
    }
}
