use core::marker::PhantomData;

/// Trait for constant or dynamic workspace handling
pub trait Space<T> {
    // In the fututre with a more powerful type system
    // one may be able to put the definition of T from the trait to the function.
    // However for this to work, we would have to be able to say something like
    // "we will output an array of (any) T", which is not yet easily possible.

    /// The workspace given, this should be an array or a vector (AsMut<[T]>)
    type Output;
    /// Returns the length of the workspace given.
    fn len(&self) -> usize;
    /// The workspace itself.
    fn workspace(&self) -> Self::Output;
}

/// Struct handles workspace while in compilation
pub struct ConstSpace<T,const N: usize>{
    phantom: PhantomData<*const T>
}

impl<T,const N: usize> Space<T> for ConstSpace<T,N>
where T: Default + Copy
{
    type Output = [T;N];
    fn len(&self) -> usize {
        N
    }
    fn workspace(&self) -> Self::Output {
        [Default::default(); N]
    }
}

impl<T, const N: usize> ConstSpace<T,N>{
    pub fn new() -> Self {
        ConstSpace {
            phantom: PhantomData
        }
    }
}

/// Struct handles workspace in run-time
pub struct DynSpace<T>{
    len: usize,
    phantom: PhantomData<*const T>
}

impl<T> Space<T> for DynSpace<T>
where T: Default + Copy
{
    type Output = std::vec::Vec<T>;
    fn len(&self) -> usize {
        self.len
    }
    fn workspace(&self) -> Self::Output {
        vec![Default::default(); self.len]
    }
}

impl<T> DynSpace<T>{
    pub fn new(len: usize) -> Self {
        DynSpace{
            len,
            phantom: PhantomData
        }
    }
}
