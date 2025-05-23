//! This examples show how to use a signal instead of a collection
//!
//! Enterpolation is written to be as generic as possible and using a signal
//! instead of a collection allows to define a (nearly) infinite detail-rich interpolation.

use enterpolation::{Chain, Signal, bspline::BSpline};

#[macro_use]
extern crate assert_float_eq;

// We define our own value signal which will be the basis of our (nearly) infinite curve.
struct ValueSignal {}

impl Signal<usize> for ValueSignal {
    type Output = f64;
    fn eval(&self, input: usize) -> f64 {
        // In Reality we would want to use a hash-like function
        // otherwise we don't create noise but a simple pattern
        (input % 10) as f64
    }
}

impl Chain for ValueSignal {
    // We can generator for any value of usize, however, we only want to use 10,000 values
    fn len(&self) -> usize {
        // Values near usize::MAX may force the spline to panic as a spline needs more knots than elements
        10_001
    }
}

fn main() {
    let spline = BSpline::builder()
        .clamped()
        .elements(ValueSignal {})
        // we use equidistant as we don't want to save that many knots in a collection
        // as alternative one could also define a KnotSignal
        .equidistant()
        .degree(3)
        // We want each knot to lie on an integer.
        .distance(0.0, 1.0)
        .constant::<4>()
        .build()
        .expect("As the curve is hardcoded, this should always work");
    // As our ValueSignal is so boring and we constructed the domain such that every knot
    // lands on an integer we know that values at x and x+10.0 are equal.
    let samples = [3.3, 157.23, 989.98];
    for sample in samples.iter() {
        assert_f64_near!(spline.eval(sample), spline.eval(sample + 10.0), 10);
    }
    println!("Nearly infinite long curve generated!");
}
