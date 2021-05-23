//TODO: For now, because of the wrapper, we want to implement interpolations with
//TODO: impl Into<E> where E: Generator<T>
//TODO: impl Into<K> where K: SortedList<R>

//TODO: Stepper is nothing else then Equidistant! Such one can use Equidistant as motor for Stepper!
//TODO: also make it/them such they can go to a custom domainscale (they should still start at 0 for ease of use)
//TODO: refactor Stepper
//TODO: create derives for Interpolation and Curve etc(?) -> https://github.com/rust-lang/rfcs/issues/1024
//TODO: make f64 the default input for Curves! -> this may reduce the need of structs with <f64,_,_,_>
//TODO: is Extrapolation as a marker trait also an idea?
use core::marker::PhantomData;
use core::borrow::Borrow;
use core::ops::{Range, Mul, Sub, Div};
use num_traits::real::Real;
use num_traits::FromPrimitive;

/// Trait which symbolises the generation or copying of an element.
///
/// This trait is fairly similar to core::ops::Index, however it does not retrurn a reference but
/// the element itself.
pub trait Generator<Input> {
    /// The element outputted
    type Output;
    /// Function to generate the element
    fn get(&self, input: Input) -> Self::Output;
    /// Helper function if one wants to sample the interpolation.
    /// It takes an iterator which yields items which are inputted into the `get` function
    /// and returns the output of the interpolation.
    fn extract<I>(&self, iterator: I) -> Extract<&Self, Self, I>
    where I: Iterator<Item = Input>
    {
        Extract {
            interpolation: self,
            iterator,
            phantom: PhantomData
        }
    }
}

/// Trait for all Interpolations.
///
/// Interpolations are nothing else then Generators which are guaranteeing that
/// the generation of elements inbetween some specific points is smooth.
pub trait Interpolation<Input> : Generator<Input>
{}

/// Curve is a specialized Interpolation which takes a real number as input
pub trait Curve<R> : Interpolation<R>
where R: Real
{
    /// The domain in which the curve uses interpolation. Not all Curves may extrapolate in a safe way.
    fn domain(&self) -> [R; 2];
    /// Takes equidistant samples of the curve (with 0.0 and 1.0 inclusive).
    fn take(&self, samples: usize) -> Take<&Self, Self, R>
    where R: FromPrimitive
    {
        Take(self.extract(Stepper::new(samples)))
    }
}

/// Iterator adaptor, which transforms an iterator with InterScalar items to an iterator with the correspondending values of the interpolation
pub struct Extract<R, P: ?Sized, I> {
    interpolation: R,
    iterator: I,
    phantom: PhantomData<*const P>
}

impl<R, P, I> Iterator for Extract<R, P, I>
where
    R: Borrow<P>,
    P: ?Sized + Interpolation<I::Item>,
    I: Iterator
{
    type Item = P::Output;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.interpolation.borrow().get(self.iterator.next()?))
    }
}

/// Newtype Take to encapsulate implementation details of the curve method take
pub struct Take<Rf, C, Re>(Extract<Rf, C, Stepper<Re>>)
where
    C: ?Sized,
    Re: Real;

impl<Rf, C, Re> Iterator for Take<Rf, C, Re>
where
    Rf: Borrow<C>,
    C: ?Sized + Curve<Re>,
    Re: Real + FromPrimitive,
{
    type Item = C::Output;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

//TODO: Stepper is nothing else then Equidistant.extract(Range) [Extract<Generator,Range>]
// and equidistant would generate the values!
/// Iterator which steps from 0.0 to 1.0 in a specific amount of constant steps.
pub struct Stepper<R: Real = f64> {
    current: usize,
    amount: usize,
    inverse_amount: R,
}

impl<R> Stepper<R>
where
    R: Real + FromPrimitive,
{
    /// Creates a new Stepper stepping from 0 to 1
    /// The given generic real number has to be able to be created from usize
    /// Also the given steps are not allowed to be less than 1
    pub fn new(steps: usize) -> Self {
        Stepper {
            current: 0,
            amount: steps - 1,
            inverse_amount: R::from_usize(steps - 1).unwrap().recip()
        }
    }
}

impl<R> Iterator for Stepper<R>
where R: Real + FromPrimitive,
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.amount {
            return None;
        }
        let res = self.inverse_amount * R::from_usize(self.current).unwrap();
        self.current += 1;
        Some(res)
    }
}


/// Trait which symbolises a list of sorted elements, also outputs the value itself instead of a reference.
///
/// This trait is mostly used to achieve better performance and accuracy for interpolation with equidistant knots
/// without needing an extra struct.
pub trait SortedList<T> : Generator<usize, Output = T> {
    fn len(&self) -> usize;
    fn first(&self) -> Option<T>;
    fn last(&self) -> Option<T>;

    fn empty(&self) -> bool {
        self.len() == 0
    }

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
struct Equidistant<R>{
    len: usize,
    step: R,
}

impl<T> Generator<usize> for Equidistant<T>
where T: Mul<usize, Output = T> + Copy
{
    type Output = T;
    fn get(&self, input: usize) -> T {
        self.step * input
    }
}

impl<R> SortedList<R> for Equidistant<R>
where R: Real + FromPrimitive + Mul<usize, Output = R> + Copy
{
    fn len(&self) -> usize {
        self.len
    }

    fn first(&self) -> Option<R> {
        if self.empty(){
            return None;
        }
        Some(self.get(0))
    }

    fn last(&self) -> Option<R> {
        if self.empty(){
            return None;
        }
        Some(self.get(self.len - 1))
    }

    fn upper_border(&self, element: R) -> [usize; 2]
    where R: PartialOrd + Copy
    {
        let scaled = element * R::from_usize(self.len()-1).unwrap();
        let min_index = scaled.floor().to_usize().unwrap().max(0);
        let max_index = scaled.ceil().to_usize().unwrap().min(self.len()-1);
        [min_index, max_index]
    }

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


/// Wrapper for struct which implement AsRef<[T]>
/// such that we are able to implement the `Generator` trait for them.
/// In the future, one may be able to disregard this and implement the trait without this wrapper
#[doc(hidden)]
struct CollectionWrapper<E,T>
(
    E,
    PhantomData<T>,
);

/// Wrapper for a struct which implements AsRef<[T]>
/// such that we are able to use it as a Generator and/or SortedList.
/// As this conversion is always working, we do not test if the collection is really sorted.
/// Such make sure the collection is sorted if you use it as a SortedList.
impl<E,T> From<E> for CollectionWrapper<E,T>
where E: AsRef<[T]>
{
    fn from(col: E) -> Self {
        CollectionWrapper(col, PhantomData)
    }
}

//TODO: instead of (), the never type COULD be an option,as we can deconstruct tuples and array through
//TODO: https://doc.rust-lang.org/reference/patterns.html#rest-patterns
/// Implementation of Collection as generator for all elements.
#[doc(hidden)]
impl<E,T> Generator<()> for CollectionWrapper<E,T>
where
    E: AsRef<[T]> + ToOwned,
    <E as ToOwned>::Owned: AsMut<[T]>,
{
    type Output = <E as ToOwned>::Owned;
    fn get(&self, _input: ()) -> Self::Output {
        self.0.to_owned()
    }
}

/// Implementation of Collection as generator for a range of elements.
/// As we do not know the range beforehand, we cannot generate an array.
/// We could allocate memory, but instead we use the fact that the range cannot be bigger than the collection itself.
#[doc(hidden)]
impl<E,T> Generator<Range<usize>> for CollectionWrapper<E,T>
where
    E: AsRef<[T]> +ToOwned,
    <E as ToOwned>::Owned: AsMut<[T]>,
{
    type Output = (<E as ToOwned>::Owned, Range<usize>);
    fn get(&self, input: Range<usize>) -> Self::Output {
        (self.0.to_owned(), input)
    }
}

/// Implementation of Collection as generator for a specific element.
#[doc(hidden)]
impl<E,T> Generator<usize> for CollectionWrapper<E,T>
where
    E: AsRef<[T]>,
    T: Copy
{
    type Output = T;
    fn get(&self, input: usize) -> Self::Output {
        self.0.as_ref()[input]
    }
}

impl<K,R> SortedList<R> for CollectionWrapper<K,R>
where
    K: AsRef<[R]>,
    R: Copy
{
    fn len(&self) -> usize{
        self.0.as_ref().len()
    }
    fn first(&self) -> Option<R>{
        self.0.as_ref().first().map(|reference| *reference)
    }
    fn last(&self) -> Option<R>{
        self.0.as_ref().last().map(|reference| *reference)
    }
}
