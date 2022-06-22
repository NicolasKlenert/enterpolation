use enterpolation::linear::Linear;
use enterpolation::Generator;

/// This mimics R's approx function
/// ```R
/// R> approx(c(0.1, 0.2, 1.0, 10.0),c(0, 1, 2,3), 1:10)$y
///  # -> 2.000000 2.111111 2.222222 2.333333 2.444444 2.555556 2.666667 2.777778 2.888889 3.000000
/// ```
fn approx(xs: Vec<f64>, ys: Vec<f64>, out_xs: Vec<f64>) -> Vec<f64> {
    let linear = Linear::builder().elements(ys).knots(xs).build().unwrap();
    linear.sample(out_xs).collect()
}

fn main() {
    let xs = vec![0.1, 0.2, 1.0, 10.0];
    let ys = vec![0.0, 1.0, 2.0, 3.0];

    let samples = (1..11).map(|x| x as f64).collect();
    let res = approx(xs, ys, samples);
    println!("{:?}", res); 
    // -> [2.0, 2.111111111111111, 2.2222222222222223, 2.3333333333333335, 2.4444444444444446, 2.5555555555555554, 2.666666666666667, 2.7777777777777777, 2.888888888888889, 3.0]
}
