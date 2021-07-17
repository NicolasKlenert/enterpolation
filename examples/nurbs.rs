//! Let's create a unit circle, as a unit circle is difficult for non-rational curves (bsplines)
//! but are surprisingly easy to do with NURBS.
//!
//! We use the data from the [wikipedia article](https://en.wikipedia.org/wiki/Non-uniform_rational_B-spline#Example:_a_circle) of NURBS,
//! only the knot vector is scaled such that the domain is from 0.0 to 4.0 insteaf of 0.0 to 2π.

use core::ops::{Add, Sub, Mul, Div};
use core::f64::consts::PI;
use enterpolation::{Generator,Curve,bspline::BSpline};
// used to test equality of f64s
use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near,
        assert_float_absolute_eq, afe_is_absolute_eq, afe_abs, afe_absolute_error_msg};

/// We create our own 2D Point
#[derive(Debug, Copy, Clone)]
pub struct Point {
    x: f64,
    y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Point {
            x,
            y,
        }
    }
    /// The squared distance of the point to the origin.
    pub fn norm(self) -> f64 {
        self.x * self.x + self.y * self.y
    }
    /// The squared distance to the other point given.
    pub fn dist(self, rhs: Point) -> f64{
        (self - rhs).norm()
    }
}

/// To use interpolation we need to define the add operation with itself.
impl Add for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

/// To calculate the distance of one point to another, we use substraction.
impl Sub for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

/// To use interpolation we need to define the multiplication with a scalar.
impl Mul<f64> for Point {
    type Output = Point;
    fn mul(self, rhs: f64) -> Self::Output {
        Point {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/// To use bezier or bsplines, we need to define a default.
impl Default for Point {
    fn default() -> Self {
        Point {
            x: 0.0,
            y: 0.0,
        }
    }
}

/// To use weights, we also need to define division with a scalar.
impl Div<f64> for Point {
    type Output = Point;
    fn div(self, rhs: f64) -> Self::Output {
        Point {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

fn main() {
    let weight = 2.0f64.sqrt() / 2.0;
    let points_with_weights = [
        (Point::new(1.0,0.0),1.0),
        (Point::new(1.0,1.0),weight),
        (Point::new(0.0,1.0),1.0),
        (Point::new(-1.0,1.0),weight),
        (Point::new(-1.0,0.0),1.0),
        (Point::new(-1.0,-1.0),weight),
        (Point::new(0.0,-1.0),1.0),
        (Point::new(1.0,-1.0),weight),
        (Point::new(1.0,0.0),1.0),
    ];
    let knots = [0.0,0.0,1.0,1.0,2.0,2.0,3.0,3.0,4.0,4.0];
    // expects are fine as we hardcoded the data.
    let nurbs = BSpline::builder()
        .elements_with_weights(points_with_weights)
        .knots(knots)
        // we know the degree of the curve at compile time, so we use constant (knots.len() - points.len())
        .constant::<3>()
        .build().expect("As the curve is hardcoded, this should always work");
    // let us test if our curve is really a unit circle!
    for val in nurbs.take(32){
        assert_f64_near!(val.norm(), 1.0);
    }
    // the speed around the circle is not constant (which is impossible to do exactly)
    // but at 0.0, 1.0, 2.0, 3.0 and 4.0 it coincides
    for val in [0.0f64,1.0,2.0,3.0,4.0].iter().copied() {
        // scale value to the corresponding circumference
        let circle_point = Point::new((val * 0.5 * PI).cos(),(val * 0.5 * PI).sin());
        assert_float_absolute_eq!(nurbs.gen(val).dist(circle_point),0.0);
    }
    println!("Successful creation of unit circle with a NURBS!");
    // but we can approximate it by linearizing.
}
