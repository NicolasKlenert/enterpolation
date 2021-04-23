/// Calculate a pascalsche triangle with the given closure until the maximal steps as levels are reached.
/// If one wants to fold all values into the first position of the given buffer
/// a step size of the length of the buffer should be used.
pub fn triangle_folding_inline<P,T>(mut triangle: P, func: impl Fn(T,T) -> T, steps: usize)
where
    P: AsMut<[T]>,
    T: Copy
{
    let elements = triangle.as_mut();
    for k in 1..steps {
        for i in 0..steps-k {
            elements[i] = func(elements[i], elements[i+1]);
        }
    }
}

pub fn lower_triangle_folding_inline<P,T>(mut triangle: P, func: impl Fn(T,T) -> T, steps: usize)
where
    P: AsMut<[T]>,
    T: Copy
{
    let elements = triangle.as_mut();
    for k in 1..=steps {
        for i in k..steps {
            elements[i] = func(elements[i-1], elements[i]);
        }
    }
}

/// Calculate a pascalsche triangle with the given closure. Degree is the number of levels one should iterate upon.
/// The given buffer triangle has to have at least a length of 1 + ... + degree + (degree+1).
/// If the given buffer is to small, this function will panic.
/// The index of the last value (the end result) will be returned.
pub fn triangle_folding<P,T>(mut triangle: P, func: impl Fn(T,T) -> T, degree: usize) -> usize
where
    P: AsMut<[T]>,
    T: Copy
{
    let elements = triangle.as_mut();
    let mut counter = 0;
    for k in (1..degree).rev(){
        for _ in 0..k{
            elements[counter + k + 1] = func(elements[counter], elements[counter+1]);
            counter += 1;
        }
        counter +=1;
    }
    counter
}
