//! This example shall illustrate bsplines and how to corrolate to other curves.

use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
use enterpolation::{bezier::Bezier, bspline::BSpline, linear::Linear, Curve};

fn main() {
    // ---- LINEAR -----
    let linear = Linear::builder()
        .elements([1.0, 3.0, 10.0, 100.0])
        .equidistant::<f64>()
        .normalized()
        .build()
        .expect("hardcoded");
    // All linear cuves can be constructed using bsplines. They work exactly the same.
    let linear_bspline = BSpline::builder()
        .elements([1.0, 3.0, 10.0, 100.0])
        .equidistant::<f64>()
        .degree(1) // We want the BSpline to be linear
        .normalized()
        .constant::<2>() // degree + 1
        .build()
        .expect("hardcoded");
    for (val1, val2) in linear.take(10).zip(linear_bspline.take(10)) {
        assert_f64_near!(val1, val2);
    }
    // ---- BEZIER -----
    let bezier = Bezier::builder()
        .elements([1.0, 3.0, 10.0, 100.0])
        .normalized::<f64>()
        .constant()
        .build()
        .expect("hardcoded");
    // All bezier curves can be constructed using clamped bsplines.
    let bezier_bspline = BSpline::builder()
        .clamped()
        .elements([1.0, 3.0, 10.0, 100.0])
        .knots(bezier.domain()) // only two knots, the start and end point, usually 0.0 and 1.0
        .constant::<4>() // number of elements
        .build()
        .expect("hardcoded");
    for (val1, val2) in bezier.take(10).zip(bezier_bspline.take(10)) {
        assert_f64_near!(val1, val2);
    }
    println!("BSplines are truly an extension of linear and bezier curves!");
}
