# Enterpolation

A library for creating and computing interpolations, extrapolations and smoothing of generic data points.

Enterpolation is a library to generate and use different interpolation and extrapolation methods. This includes linear interpolation, bezier curves, B-spline and their weighted and non-uniform counterparts like NURBS. This library aims to be as generic as possible to allow interpolating elements of any vector space. Also building and using different interpolations should create as less friction as possible such that one may experiment with different methods to find the interpolation which best fits their needs. To achieve this, Enterpolation heavily uses consistent builder options to allow customization and minimise the need to change code while experimenting.

## Table of Contents

- [Usage](#usage)
- [Crate Features](#crate-features)
- [Details](#details)
- [Contributing](#contributing)
- [License](#license)

## Usage

Add this to your `Cargo.toml`:
```toml
[dependencies]
enterpolation = "0.1.1"
```

Here is a simple example creating a linear interpolation of `f64` and extracting 21 values from start to end. This library supports all elements which can be added together and multiplied with a scalar (in our case also `f64`). Instead of interpolating floats, one could interpolate coordinates, rotations, transformations, velocities, geometries, sound, colors and so on.
```rust
use enterpolation::{linear::{Linear, LinearError}, Curve};

fn main() -> Result<(), LinearError> {
  let lin = Linear::builder()
    .elements([0.0,5.0,-5.0,0.0])
    .knots([0.0,0.2,0.8,1.0])
    .build()?;
  // generate and print 21 values with equal distances
  // that is: 0.0, 0.05, 0.10, ..., 0.95, 1.00
  for value in lin.take(21){
    println!("{:?}",value);
  }
  Ok(())
}
```

Another example shows how to create the 1D cardinal cubic B-spline example shown on [Wikipedia's B-splines page](https://en.wikipedia.org/wiki/B-spline) in several different ways.

```rust
use enterpolation::{bspline::{BSpline, BSplineError}, Curve};
use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};

fn main() -> Result<(), BSplineError> {
  let bspline = BSpline::builder()
        .clamped()             // the curve should be clamped (variation)
        .elements([0.0,0.0,0.0,6.0,0.0,0.0,0.0])
        .equidistant::<f64>() // knots should be evenly distributed
        .degree(3)            // cubic curve
        .domain(-2.0,2.0)     // input domain
        .constant::<4>()      // we need degree+1 space to interpolate
        .build()?;
  let same_but_different = BSpline::builder()
        .elements([0.0,0.0,0.0,6.0,0.0,0.0,0.0])
        // we repeat the end and start knot 3 times, as
        // we have a degree of 3 and we want a clamped curve
        .knots([-2.0,-2.0,-2.0,-1.0,0.0,1.0,2.0,2.0,2.0])
        .constant::<4>()     // we need knots.len() - elements.len() + 2
        .build()?;
  let run_time_spline = BSpline::builder()
        .elements(vec![0.0,0.0,0.0,6.0,0.0,0.0,0.0])
        .knots(vec![-2.0,-2.0,-2.0,-1.0,0.0,1.0,2.0,2.0,2.0])
        .dynamic()
        .build()?;
  // all three construction define the same curve
  for (a,b,c) in bspline.take(10)
  .zip(same_but_different.take(10))
  .zip(run_time_spline.take(10))
  .map(|((a,b),c)| (a,b,c)) {
    assert_f64_near!(a,b);
    assert_f64_near!(b,c);
  }
  Ok(())
}
```

For further information how to use any curve, one may look at the main traits of this crate: [`Generator`] and [`Curve`].

[`Generator`]: https://docs.rs/enterpolation/0.1.0/enterpolation/trait.Generator.html
[`Curve`]: https://docs.rs/enterpolation/0.1.0/enterpolation/trait.Curve.html

### Further Examples

Examples showcasing use cases (like defining a smooth color gradient or using NURBS) can be found in the [examples] directory.

[examples]: https://github.com/NicolasKlenert/enterpolation/tree/main/examples

## Crate Features

This crate comes with a feature for every different interpolation method, such allowing to only include the necessary modules. All features are enabled by default.

If one wants to only enable specific crate features, they have to use the following `Cargo.toml` dependency configuration:
```toml
[dependencies.enterpolation]
version = "0.1"
default-features = false
# re-enable all wanted features
features = ["linear"]
```

- **std** - When enabled, run-time allocations may be done with std::vec. For the most part one can disable this feature and implement the necessary traits for their custom run-time allocation or only use arrays.
- **libm** - This feature has to be enabled for the crate to work properly if the **std** feature is disabled.
- **linear** - Enables all relevant methods and the construction of linear interpolation.
- **bezier** - Enables all relevant methods and the construction of bezier curves.
- **bspline** - Enables all relevant methods and the construction of B-Spline.

## Details

#### Difference to other Crates

Enterpolation aims to be a crate which generalizes curves to an extend in which you will be able to change the type of curve without changing too much of your code. It provides a simple interface to take samples from your curve. Its focus on ease of use also differentiate itself from other crate:
- Builder patterns check your input for you and return an error explaining what did not work.
- A wide array of different input types are allowed, such no conversions are needed. It also allows to optimize the curve for your needs.
- Everything which may be interpolated can be interpolated into a curve.
- Each sampling input will return a value; no panics occur. Instead the curve will try to extrapolate (which may not be numerically stable). If one wants to clamp values instead, adaptors may be used.
- Many adaptors allow further changes of the curve which would not be otherwise possible.

This crate is _not_ a graphics library. Good crates which are used in the graphical context exist already. This crate aims to be more general to allow experimentations and cover edge cases for which these crates do not fit.

#### Performance

Measuring performance can be rather tricky and optimizations of the compiler can be rather chaotically (in the sense that performance of a function may change even if the function itself was not changed), such these measurements should be taken with a grain of salt. They are only here to guide a decision. Performance changes of +-15% are not rare, such in these regions we consider functions to be more or less equally fast.

We compare dynamic bsplines with their equivalent bsplines in the crate `bspline` (v1.0.0). Other than that, we also compare bsplines with constant elements and knots, bsplines with shared elements and knots and bsplines with uniform knots (and constant elements). The bspline tested have 100 elements and degree 3.

| BSplines            | dynamic   | constant  | static    | uniform   | crate bspline |
| ------------------- | --------- | --------- | --------- | --------- | ------------- |
| sampling 200 values | 23.160 us | 6.6834 us | 6.6521 us | 9.9186 us | 20.734 us     |
| creation of curve   | 389.72 ns | 389.23 ns | 225.27 ns | 131.35 ns | 496.66 ns     |

The sampling of uniform curves will be faster in comparison to other curves the more knots a curve has.

#### Requirements for Elements

If the elements you want to interpolate already implement [addition] with themselves and [multiplication] with a scalar, you should already be fine. If that is not the case, you may want to consider implementing these, as most interpolations will only work properly if the elements are living in a vector-space (and such addition and multiplication is defined for them).

Otherwise this crate re-exports a trait [Merge], which represents the capability of an element to be merged with another one. This trait is necessary for all interpolations. Furthermore the core [Default] trait is also necessary for bezier curves and B-splines.

Elements can be given to the curve with an array, a vector or by implementing the [DiscreteGenerator] trait. Basically every collection with an indexing operation can implement this trait. However generators can also implement it. Such one may generate the elements which should be interpolated on-the-fly. This can reduce the memory footprint if elements can be generically generated and one wants to interpolate many elements.

[addition]: https://doc.rust-lang.org/core/ops/trait.Add.html
[multiplication]: https://doc.rust-lang.org/core/ops/trait.Mul.html
[Merge]: https://docs.rs/topology-traits/0.1.1/topology_traits/trait.Merge.html
[Default]: https://doc.rust-lang.org/beta/core/default/trait.Default.html
[DiscreteGenerator]: https://docs.rs/enterpolation/0.1.0/enterpolation/trait.DiscreteGenerator.html

#### Requirements for Knots

Knots represent the location of the elements in the input space. Such knots are usually of the same type as your input for the interpolation itself. As all interpolations (yet) are curves, usually knots are `f32` or `f64`. Elements must be multipliable with knots and knots have to be sorted with the smallest knot at index zero.

Knots also can be given via an array or a vector, or some other type which implements the [`DiscreteGenerator`] trait. One may also implement the [`SortedGenerator`] trait if the type is always guaranteed to represent sorted knots.

[`DiscreteGenerator`]: https://docs.rs/enterpolation/0.1.0/enterpolation/trait.DiscreteGenerator.html
[`SortedGenerator`]: https://docs.rs/enterpolation/0.1.0/enterpolation/trait.SortedGenerator.html

#### B-spline Peculiarity

Except for the *Legacy* mode of B-splines, the construction in this crate works different than in most other libraries. Normally the first and last knots of each B-spline definition are useless, as the don't effect the generation of points inside the allowed domain of a B-spline. Without these two knots it's more clear how B-spline operate and in which way they are similar to the other interpolation curves. Such the decision was made to forego the usual definition. As we acknowledge that there may be a need of accepting the old format of knots, `BSpline` implements a legacy mode which can be used to define a B-spline in an old fashioned way.

The example [bspline-reasoning] illustrates the elegance of forgetting these two end knots and contains code which makes use of the legacy mode.

[bspline-reasoning]: https://github.com/NicolasKlenert/enterpolation/blob/main/examples/bspline_reasoning.rs

#### B-spline Variations

As B-splines are rather complex curves, their [builder] allows different modes to make it possible to define the curves how the user would like to.

- **open** - This can be seen as the default mode of the builder. No guarantees are made regarding the shape of the curve. With a correct configuration, this mode is able to achieve the same curve as any other mode.
- **clamped** - This mode clamps the curve such that its start- and endpoint are guaranteed to be the first and last element given. This is done by repeating the first and last knots.
- **legacy** - This mode may be used to configure a B-spline the same way most other sources do. This mode is useful if one only gets the values for the configuration of a B-spline and is not creating them themselves.

[builder]: bspline::BSplineBuilder

## Contributing

All contributions are welcome, no matter how huge or tiny. If you are interested, take a look at [CONTRIBUTING.md](https://github.com/NicolasKlenert/enterpolation/blob/main/CONTRIBUTING.md) for guidelines.

## License

Licensed under either of

* [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
* [MIT license](http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
