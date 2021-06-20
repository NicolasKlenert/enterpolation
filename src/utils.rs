//! Module for different utilities which are used across other modules or to help the user of the library.
use core::cmp::Ordering;
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

/// Find the indices to the corresponding elements inside the collection
/// for which the given element is inbetween.
/// We assume that the collection is non-empty and ordered, to use binary search.
/// Elements in the collections which are exactly equal to the element are ignored.
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
/// # use enterpolation::utils::border;
/// let arr = [0.0,0.1,0.2,0.7,0.7,0.7,0.8,1.0];
/// assert_eq!(border(&arr,-1.0),(0,1));
/// assert_eq!(border(&arr,0.15),(1,2));
/// assert_eq!(border(&arr,0.7),(2,6));
/// assert_eq!(border(&arr,20.0),(6,7));
/// ```
pub fn border<T>(collection: &[T], element: T) -> (usize, usize)
where
    T: PartialOrd + Copy
{
    let mut min_index = 0;
    let mut max_index = collection.len() - 1;
    while max_index - min_index > 1 {
        let index = min_index + (max_index - min_index) / 2;
        let sample = collection[index];

        match element.partial_cmp(&sample){
            // not comparable
            None => panic!("Non comparable elements found in function border"),
            Some(Ordering::Less) => max_index = index,
            Some(Ordering::Equal) => return (
                lower_border(&collection[min_index..max_index], element).0,
                upper_border(&collection[min_index..max_index], element).1
            ),
            Some(Ordering::Greater) => min_index = index,
        }
    }
    (min_index, max_index)
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
pub fn lower_bound<T>(collection: &[T], element: T) -> usize
where T: PartialOrd + Copy
{
    if *collection.last().unwrap() <= element {
        return collection.len() - 1;
    }
    upper_border(collection, element).0
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
pub fn upper_bound<T>(collection: &[T], element: T) -> usize
where T: PartialOrd + Copy
{
    if *collection.first().unwrap() >= element {
        return 0;
    }
    lower_border(collection, element).1
}

/// This function has a slightly better performance in the specific case one only needs the max_index of the function
/// upper_border. That is `strict_upper_bound(collection, element) == upper_border(collection, element).1`.
/// Also they are diferent in the edge case that if all elements in the array are smaller, this function **will** return 0.
/// `upper_border` on the other hand will return 1 (as the min_index occupies 0).
/// If all elements are bigger, this function will return len()-1.
///
/// #Panic
///
/// Panics if the collection is empty.
///
pub fn strict_upper_bound<T>(collection: &[T], element: T) -> usize
where T: PartialOrd + Copy
{
    let mut pointer = 0;
    let mut count = collection.len();
    while count > 0 {
        let step = count / 2;
        let sample = pointer + step;
        if element >= collection[sample]{
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
pub fn strict_lower_bound<T>(collection: &[T], element: T) -> usize
where T: PartialOrd + Copy
{
    let mut pointer = collection.len() - 1;
    let mut count = collection.len();
    while count > 0 {
        let step = count / 2;
        let sample = pointer - step;
        if element >= collection[sample]{
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
pub fn upper_border<T>(collection: &[T], element: T) -> (usize, usize)
where
    T: PartialOrd + Copy
{
    let mut min_index = 0;
    let mut max_index = collection.len() - 1;

    while max_index - min_index > 1 {
        let index = min_index + (max_index - min_index) / 2;
        let sample = collection[index];

        if element < sample {
            max_index = index;
        } else {
            min_index = index;
        }
    }
    (min_index, max_index)
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
pub fn lower_border<T>(collection: &[T], element: T) -> (usize, usize)
where
    T: PartialOrd + Copy
{
    let mut min_index = 0;
    let mut max_index = collection.len() - 1;

    while max_index - min_index > 1 {
        let index = min_index + (max_index - min_index) / 2;
        let sample = collection[index];

        if element > sample {
            min_index = index;
        } else {
            max_index = index;
        }
    }
    (min_index, max_index)
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

// /// Calculate a pascalsche triangle with the given closure. Degree is the number of levels one should iterate upon.
// /// The given buffer triangle has to have at least a length of 1 + ... + degree + (degree+1).
// /// If the given buffer is to small, this function will panic.
// /// The index of the last value (the end result) will be returned.
// pub fn triangle_folding<P,T>(mut triangle: P, func: impl Fn(T,T) -> T, degree: usize) -> usize
// where
//     P: AsMut<[T]>,
//     T: Copy
// {
//     let elements = triangle.as_mut();
//     let mut counter = 0;
//     for k in (1..degree).rev(){
//         for _ in 0..k{
//             elements[counter + k + 1] = func(elements[counter], elements[counter+1]);
//             counter += 1;
//         }
//         counter +=1;
//     }
//     counter
// }

#[cfg(test)]
mod test {

    #[test]
    fn upper_border() {
        // test if upper_border works with only 1 element
        let arr = [5.0];
        assert_eq!(super::upper_border(&arr,0.5), (0,0));
        // all other behaviours are checked by doc-test
    }
}
