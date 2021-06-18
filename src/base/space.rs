use core::marker::PhantomData;

/// Trait for constant or dynamic workspace handling.
///
/// We do want to generate workspaces every time the method is called as this allows as safe concurrency.
/// This may impact performance as for DynSpace we always allocate memory. However as their is an alternative,
/// this is accepted.
pub trait Space<T> {
    // In the fututre with a more powerful type system
    // one may be able to put the definition of T from the trait to the function.
    // However for this to work, we would have to be able to say something like
    // "we will output an array of (any) T", which is not yet easily possible.

    /// The workspace given, this should be an array or a vector (AsMut<[T]>)
    type Output : AsMut<[T]>;
    /// Returns the length of the workspace given.
    fn len(&self) -> usize;
    /// The workspace itself.
    fn workspace(&self) -> Self::Output;
}

/// Struct handles workspace while in compilation
#[derive(Debug, Copy, Clone)]
pub struct ConstSpace<T,const N: usize>{
    _phantom: PhantomData<*const T>,
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
    /// Create a constant worksprace at compile-time.
    pub fn new() -> Self {
        ConstSpace {
            _phantom: PhantomData
        }
    }
}

/// Struct handles workspace at run-time.
#[derive(Debug, Copy, Clone)]
pub struct DynSpace<T>{
    len: usize,
    _phantom: PhantomData<*const T>
}

impl<T> Space<T> for DynSpace<T>
where T: Default + Copy
{
    type Output = Vec<T>;
    fn len(&self) -> usize {
        self.len
    }
    fn workspace(&self) -> Self::Output {
        vec![Default::default(); self.len]
    }
}

impl<T> DynSpace<T>{
    /// Create a workspace with given length at run-time.
    pub fn new(len: usize) -> Self {
        DynSpace{
            len,
            _phantom: PhantomData
        }
    }
}
