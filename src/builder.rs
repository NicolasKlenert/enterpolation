//! Module with structures and utilities used in many builders

use core::marker::PhantomData;

// pub trait WeightMarker{}

/// Struct indicator to mark that we don't use weights
#[derive(Debug, Copy, Clone)]
pub struct WithoutWeight;

/// Struct indicator to mark that we use weights
#[derive(Debug, Copy, Clone)]
pub struct WithWeight;

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
