//TODO: also make it/them such Stepper can go to a custom domainscale (they should still start at 0 for ease of use)
//TODO: create derives for Interpolation and Curve etc(?) -> https://github.com/rust-lang/rfcs/issues/1024
//TODO: make f64 the default input for Curves! -> this may reduce the need of structs with <f64,_,_,_>
//TODO: is Extrapolation as a marker trait also an idea?
use core::marker::PhantomData;
use core::ops::Range;

mod generator;
mod list;
mod space;

pub use generator::{Generator, Interpolation, Curve, FiniteGenerator, Extract, Stepper};
pub use list::{Equidistant, ConstEquidistant, SortedList};
pub use space::{Space, DynSpace, ConstSpace};


/// Wrapper for struct which implement AsRef<[T]>
/// such that we are able to implement the `Generator` trait for them.
/// In the future, one may be able to disregard this and implement the trait without this wrapper
pub struct CollectionWrapper<E,T>
(
    E,
    PhantomData<T>,
);

impl<E,T> CollectionWrapper<E,T>{
    /// Wrap the given collection with a wrapper.
    ///
    /// This is necessary for the collections to implement the Generator trait, as Rust does not allow
    /// contraining generics only on the where clause yet.
    pub const fn new(col: E) -> Self {
        CollectionWrapper(col,PhantomData)
    }
}


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

impl<E,T> FiniteGenerator for CollectionWrapper<E,T>
where
    E: AsRef<[T]>,
    T: Copy
{
    fn len(&self) -> usize {
        self.0.as_ref().len()
    }
}

impl<K,R> SortedList<R> for CollectionWrapper<K,R>
where
    K: AsRef<[R]>,
    R: Copy
{}

// CollectionWrapper references may be used as Space for specific Bezier Curves
// impl<E,T> Space<T> for &CollectionWrapper<E,T>
// where
//     E: AsRef<[T]> + ToOwned
// {
//     type Output = E::Owned;
//     fn len(&self) -> usize {
//         self.0.as_ref().len()
//     }
//     fn workspace(&self) -> Self::Output {
//         self.0.to_owned()
//     }
// }
