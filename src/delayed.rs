use core::mem::MaybeUninit;

unsafe fn transmute_maybe_uninit_array<T, const N: usize>(mut arr: [MaybeUninit<T>;N]) -> [T;N] {
    // Using &mut as an assertion of unique "ownership"
    let res = (&mut arr as *mut _ as *mut [T; N]).read();
    core::mem::forget(arr);
    res
}

fn example(){
    // Create an uninitialized array of `MaybeUninit`, such `T` must not be copied twice.
    // The `assume_init` is safe because the type we are claiming to have initialized here is a
    // bunch of `MaybeUninit`s, which do not require initialization.
    let mut grad : [MaybeUninit<T>; K] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    // Copy every element over. Theoretically we could calulcate the first step here.
    // Before doing this, some benchmarks should be done. Optimasation is too early.
    // Dropping a `MaybeUninit` does nothing, thus using indexing assignment instead of `ptr::write` is fine.
    // if there is a panic during this loop, we have a memory leak, but there is no memory safety issue.
    for i in 0..K {
        grad[i] = MaybeUninit::new(elements.as_mut()[i]);
    }
    // Everyting is initialized. Transmuting should be fine.
    // Again this is not working because of https://github.com/rust-lang/rust/issues/61956
    // let mut grad = unsafe {mem::transmute::<_, [T;K]>(grad)};
    // use hack instead:
    let mut grad = unsafe {transmute_maybe_uninit_array(grad)};
}
