//! Module with structures and utilities used in many builders

use core::marker::PhantomData;

/// Struct indicator to mark if we use weights
#[derive(Debug, Copy, Clone)]
pub struct WithWeight<T>(pub T);

/// Struct indicator to mark information not yet given.
#[derive(Debug, Copy, Clone)]
pub struct Unknown;

/// Struct indicator to mark the wish of using equidistant knots.
#[derive(Debug, Copy, Clone)]
pub struct Output<R = f64>(PhantomData<*const R>);

impl<R> Output<R> {
    pub const fn new() -> Self {
        Output(PhantomData)
    }
}
