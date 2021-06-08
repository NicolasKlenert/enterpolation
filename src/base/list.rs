use core::marker::PhantomData;
use core::ops::{Sub, Div};
use num_traits::real::Real;
use num_traits::FromPrimitive;

use core::fmt::Debug;

pub use super::{Generator, Interpolation, Curve, DiscreteGenerator, Extract, Stepper};

// TODO: Search for "//TODO: implement all SortedGenerator functions with the underlying SortedGenerator!"
// TODO: and do this! Otherwise there is no performance boost for Equidistant as we wrap them!

// REMARK: It may be valuable to create traits SortedNonEmpty and SortedNonSingular
// REMARK: These would be Sorted + NonEmpty and Sorted + MinSize<2>.
// REMARK: They would implement the specified functions without risk of panics and possible use of the functions.
// REMARK: However this will create even more traits which have to be implemented.

// Ideas: for linear we would like a function which returns us the nearest two knots and the factor!
// Ideas: If a knot lies directly on top of the sample, just return the knot twice (or any other neighbor with it, we do not care).
// Ideas: If there is only one element, return the element and any factor!
// Ideas: If there are no elements, we are allowed to panic or anything else. 0 elements are never allowed!
// Ideas: for bspline we would like a simple function which returns us the minimal bigger knot.
// Ideas: -> if all elements are smaller then the sample, return the len of the collection
// Ideas: Also panic if no element is given!

// Summary: we want a function which returns us the bigger element when possible (or the last element)
// Summary: we want to get some border, 1 step or dynamic steps smaller. The distance between these two points have to be exact.
// Summary: If the two elements are not equal, we calculate the factor
// Summary: If the two elements are equal, we want to return 1.0 as factor
// Summary: If the distance between these two shall be greater then the size of the collection, we panic!
// --> for bspline, MinSize does not make any sense for DynSpace. For ConstSpace, it's possible
// --> for bspline and DynSpace we have to check at runtime (and have to check each time we want to remove an element)

/// Trait to mark a generator as sorted.
///
/// This guarantees that the generated elements of a generator are
/// - comparable (you could define the trait Ord for the set of all generated elements)
/// - non-strictly increasing
///
/// Also it implements default search functions which can be overriden to achieve better performance
/// and accuracy.
///
/// # Panics
///
/// Some or all of this functions *may* panic. Each function has it's own panic segment which
/// describes its needs. To guarantee no panics at runtime, one should use struct or traits which
/// guarantee the needs of the functions. The MinSize trait was created just for this.
pub trait SortedGenerator : DiscreteGenerator
{
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
    fn lower_bound(&self, element: Self::Output) -> usize
    where Self::Output: PartialOrd + Copy
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
    fn upper_bound(&self, element: Self::Output) -> usize
    where Self::Output: PartialOrd + Copy
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
    fn strict_upper_bound(&self, element: Self::Output) -> usize
    where Self::Output: PartialOrd + Copy
    {
        let mut pointer = 0;
        let mut count = self.len();
        while count > 0 {
            let step = count / 2;
            let sample = pointer + step;
            if element >= self.gen(sample){
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
    fn strict_lower_bound(&self, element: Self::Output) -> usize
    where Self::Output: PartialOrd + Copy
    {
        let mut pointer = self.len() - 1;
        let mut count = self.len();
        while count > 0 {
            let step = count / 2;
            let sample = pointer - step;
            if element >= self.gen(sample){
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
    fn upper_border(&self, element: Self::Output) -> [usize; 2]
    where
        Self::Output: PartialOrd + Copy
    {
        let mut min_index = 0;
        let mut max_index = self.len() - 1;

        while max_index - min_index > 1 {
            let index = min_index + (max_index - min_index) / 2;
            let sample = self.gen(index);

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
    fn upper_border_with_factor(&self, element: Self::Output) -> (usize, usize, Self::Output)
    where
        Self::Output: PartialOrd + Sub<Output = Self::Output> + Div<Output = Self::Output> + Copy + Debug
    {
        let [min_index, max_index] = self.upper_border(element);
        (min_index, max_index, self.factor(min_index, max_index, element))
    }

    /// Calculate the factor of `element` inbetween `min` and `max`.
    /// That is, the factor would be needed to generate `element` from a linear interpolation of
    /// `min` and `max`, with `min` being the element generated by `min_index` and the same holds for `max_index`.
    fn factor(&self, min_index: usize, max_index: usize, element: Self::Output) -> Self::Output
    where Self::Output: Sub<Output = Self::Output> + Div<Output = Self::Output> + Copy
    {
        let max = self.gen(max_index);
        let min = self.gen(min_index);
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
    fn lower_border(&self, element: Self::Output) -> [usize; 2]
    where
        Self::Output: PartialOrd + Copy
    {
        let mut min_index = 0;
        let mut max_index = self.len() - 1;

        while max_index - min_index > 1 {
            let index = min_index + (max_index - min_index) / 2;
            let sample = self.gen(index);

            if element > sample {
                min_index = index;
            } else {
                max_index = index;
            }
        }
        [min_index, max_index]
    }
}

/// Marker trait to mark a generator to have at least a length of N.
pub trait MinSizeGenerator<const N: usize> : DiscreteGenerator {}

/// Marker trait to mark a generator as non empty.
///
/// This trait is not implemented by hand. Insted of implementing this trait, implement MinSizeGenerator<1>.
/// There is no difference in these two except this implementation detail.
/// When rust allows trait alias in stable, this will be unnecassary.
pub trait NonEmptyGenerator : MinSizeGenerator<1> {
    fn first(&self) -> Self::Output {
        self.gen(0)
    }
    fn last(&self) -> Self::Output {
        self.gen(self.len() - 1)
    }
}

// Delete and make NonEmptyGenerator a trait alias when this is possible.
impl<T> NonEmptyGenerator for T where T: MinSizeGenerator<1> {}
// Do this with a macro and at some point maybe const features are strong enough to do it generically.
// This implementation conflicts with generic implementations as we can't just take the maximum value for N, but do all N.
// impl<T> MinSizeGenerator<0> for T where T: MinSizeGenerator<1> {}
// impl<T> MinSizeGenerator<1> for T where T: MinSizeGenerator<2> {}
// impl<T> MinSizeGenerator<2> for T where T: MinSizeGenerator<3> {}

/// Struct to represent a collection/generator with a minimum size of N.
#[derive(Copy,Clone,Eq,PartialEq,Ord,PartialOrd,Hash,Debug)]
pub struct MinSize<C,const N: usize>(C);

impl<C,const N: usize> MinSize<C,N>
where C: DiscreteGenerator
{
    /// Returns Some(col) if collection has a minimum size of `N`, otherwise returns None.
    pub fn new(col: C) -> Option<Self>{
        if col.len() < N {
            return None;
        }
        Some(MinSize(col))
    }
}

// We are not able to implement `From<MinSize<C,N>> for C` because it might conflict with
// `impl<C> From<MinSize<C,N>> for LocalType` in another crate, which is always allowed.
// See https://stackoverflow.com/questions/63119000/why-am-i-required-to-cover-t-in-impl-foreigntraitlocaltype-for-t-e0210
impl<C,const N: usize> MinSize<C,N> {
    /// Creates a minimal size collection without checking if it has at least this size.
    ///
    /// As too few elements are not going to create UB, this functin is safe.
    /// However at some point something will probably panic.
    pub const fn new_unchecked(col: C) -> Self{
        MinSize(col)
    }

    /// Returns the inner collection
    pub fn get(self) -> C {
        self.0
    }
}

impl<C,const N: usize> Generator<usize> for MinSize<C,N>
where C: Generator<usize>
{
    type Output = C::Output;
    fn gen(&self, input: usize) -> Self::Output {
        self.0.gen(input)
    }
}

impl<C,const N: usize> DiscreteGenerator for MinSize<C,N>
where C: DiscreteGenerator
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<C: DiscreteGenerator,const N: usize> MinSizeGenerator<N> for MinSize<C,N> {}
impl<C: SortedGenerator,const N: usize> SortedGenerator for MinSize<C,N> {
    //TODO: implement all SortedGenerator functions with the underlying SortedGenerator!
    fn upper_border_with_factor(&self, element: Self::Output) -> (usize, usize, Self::Output)
    where
        Self::Output: PartialOrd + Sub<Output = Self::Output> + Div<Output = Self::Output> + Copy + Debug
    {
        self.0.upper_border_with_factor(element)
    }
}

/// Type alias which is more telling.
pub type NonEmpty<C> = MinSize<C,1>;

/// Struct to represent a sorted collection/generator.
pub struct Sorted<C>(C);

impl<C> Sorted<C>
where
    C: DiscreteGenerator,
    C::Output: PartialOrd
{
    /// Returns Some(Sorted) if collection is sorted, otherwise returns None
    pub fn new(col: C) -> Option<Self>{
        if col.is_empty() {
            return Some(Sorted(col))
        }
        let mut last = col.gen(0);
        for i in 1..col.len(){
            let current = col.gen(i);
            if !(last <= current){
                return None;
            }
            last = current;
        }
        Some(Sorted(col))
    }
}

impl<C> Sorted<C>{
    /// Creates a sorted collection without checking if it is sorted.
    ///
    /// As unsorted collection will not create UB but will probably panic at some point,
    /// such this function is still safe, even if an unsorted collection is given.
    pub const fn new_unchecked(col: C) -> Self{
        Sorted(col)
    }
}

impl<C> Generator<usize> for Sorted<C>
where C: Generator<usize>
{
    type Output = C::Output;
    fn gen(&self, input: usize) -> Self::Output{
        self.0.gen(input)
    }
}

impl<C> DiscreteGenerator for Sorted<C>
where C: DiscreteGenerator
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<C: DiscreteGenerator> SortedGenerator for Sorted<C> {}
impl<C: MinSizeGenerator<N>, const N: usize> MinSizeGenerator<N> for Sorted<C> {}

/// Struct used as a generator for equidistant elements.
/// Acts like an array of knots.
pub struct Equidistant<R = f64>{
    len: usize,
    step: R,
    offset: R,
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
    /// Create a generator for equidistant real numbers with `len` steps from 0.0 to 1.0.
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

    /// Create a generator for equidistant real numbers with `len` steps from `start` to `end`.
    ///
    /// #Panics
    ///
    /// Panics if the given length is 0 or `length -  1` can not be transformed into R.
    pub fn new(start: R, end: R, len: usize) -> Self {
        Equidistant {
            len,
            step: (end - start) / R::from_usize(len - 1).unwrap(),
            offset: start
        }
    }
}

impl<R> Generator<usize> for Equidistant<R>
where R: Real + FromPrimitive
{
    type Output = R;
    fn gen(&self, input: usize) -> R {
        self.step * R::from_usize(input).unwrap() + self.offset
    }
}

impl<R> DiscreteGenerator for Equidistant<R>
where R: Real + FromPrimitive
{
    fn len(&self) -> usize {
        self.len
    }
}

//TODO: Test upper_border_with_factor for all implementations -> collection, equidistant and ConstEquidistant!
//TODO: Returning an Option or such would be more idiomatic! -> what to do with 0 or 1 element?!
//TODO: upper_border is difficult to write for equidistant (making sure both indices are valid but are not the same!)
//TODO: we don't even have a contract for that, such we should think about it carefully!
//TODO: It is important to note that upper_border_with_factor does not act like upper_border -> Change the name!

impl<R> SortedGenerator for Equidistant<R>
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
        let scaled = element / self.step;
        let min_index = scaled.floor().to_usize().unwrap().max(0);
        let max_index = scaled.ceil().to_usize().unwrap().min(self.len()-1);
        let factor = scaled.fract();
        (min_index, max_index, factor)
    }
}
impl<R> MinSizeGenerator<1> for Equidistant<R> where R: Real + FromPrimitive {}

/// Struct used as a generator for equidistant elements in constant context.
/// Acts like an array of knots.
///
/// This struct is necessary as to date neither generic bounds nor floating point opterations are
/// allowed in constant functions. Such to be able to use Equidistant in a constant context,
/// we use this structure instead.
///
/// In comparison to `Equidistant`, this struct is slower (as it has to do more calculations) and
/// only represents knots in [0.0,1.0]. However as knot base for interpolations, it is more performant,
/// as we have the knowledge of the domain.
pub struct ConstEquidistant<R = f64>{
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
    fn gen(&self, input: usize) -> R {
        R::from_usize(input).unwrap() / R::from_usize(self.len - 1).unwrap()
    }
}

impl<R> DiscreteGenerator for ConstEquidistant<R>
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

impl<R> SortedGenerator for ConstEquidistant<R>
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
        R: PartialOrd + Sub<Output = R> + Div<Output = R> + Copy + Debug
    {
        let scaled = element * R::from_usize(self.len()-1).unwrap();
        let min_index = scaled.floor().to_usize().unwrap().max(0);
        let max_index = scaled.ceil().to_usize().unwrap().min(self.len()-1);
        let factor = scaled.fract();
        (min_index, max_index, factor)
    }
}
