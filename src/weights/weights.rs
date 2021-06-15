//! This module is a generator adaptor to transform generators such that they output homogeneous data.

pub struct Weights<G> {
    gen: G,
}

// G should have (T,R) -> Homogeneous<T,R>
// as we can generate (T,R) from T and R by just stacking them! 
