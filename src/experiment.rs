use core::marker::PhantomData;

// Scalar which represents an inbetween two points (usually between 0.0 and 1.0) (from and to constants?!)
type InterScalar = f64;

//this is a test sruct, the mother of all interpolations -> every other interpolation is just a specific InterpolationFrankenstein
/// S stands for the scalar(component) which we get as input
/// K are the knots, identifier for the rest
/// E are the elements and weights identified by the knots
/// I is the Interpolation itself (takes scalar, elements and weights and returns the output)
pub struct InterpolationFrankenstein<S, K, E, I>
where
    S: Scalar,
    K: Knots,
    E: Samples<Input = K::Output>,
    I: InterpolationFunction<Construct = E>,
{
    scalar: PhantomData<S>,
    knots: K,
    elements: E,
    interpolation: I,
}

/// E are the elements (and weights) the interpolationfunction can interpolate
pub trait InterpolationFunction {
    type Construct: Samples;
    type Output;
    fn interpolate(elements: <Self::Construct as Samples>::Output) -> Self::Output;
}

pub trait Scalar {}

pub trait Samples {
    type Input;
    type Output;
}

pub trait Knots {
    type Output;
}
