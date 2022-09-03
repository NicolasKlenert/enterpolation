use criterion::{black_box, criterion_group, criterion_main, Criterion};
use enterpolation::bspline::BSpline;
use enterpolation::{Curve, Generator, Stepper};

const ELEMENTS: [f64; 100] = [
    943.0, 978.0, 579.0, 15.0, 608.0, 938.0, 669.0, 98.0, 720.0, 303.0, 345.0, 421.0, 767.0, 798.0,
    379.0, 379.0, 794.0, 774.0, 559.0, 315.0, 141.0, 465.0, 721.0, 398.0, 265.0, 921.0, 664.0,
    701.0, 433.0, 494.0, 286.0, 331.0, 976.0, 493.0, 296.0, 93.0, 963.0, 422.0, 398.0, 358.0,
    906.0, 80.0, 272.0, 287.0, 867.0, 995.0, 109.0, 874.0, 840.0, 931.0, 554.0, 405.0, 643.0,
    820.0, 229.0, 736.0, 117.0, 317.0, 105.0, 168.0, 16.0, 267.0, 481.0, 547.0, 308.0, 463.0,
    664.0, 948.0, 655.0, 201.0, 439.0, 910.0, 431.0, 945.0, 441.0, 682.0, 272.0, 716.0, 851.0,
    916.0, 98.0, 162.0, 536.0, 129.0, 369.0, 753.0, 735.0, 511.0, 346.0, 137.0, 221.0, 715.0,
    565.0, 305.0, 649.0, 913.0, 215.0, 713.0, 321.0, 95.0,
];
const KNOTS: [f64; 102] = [
    0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0, 31.0,
    32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0, 41.0, 42.0, 43.0, 44.0, 45.0, 46.0, 47.0,
    48.0, 49.0, 50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 60.0, 61.0, 62.0, 63.0,
    64.0, 65.0, 66.0, 67.0, 68.0, 69.0, 70.0, 71.0, 72.0, 73.0, 74.0, 75.0, 76.0, 77.0, 78.0, 79.0,
    80.0, 81.0, 82.0, 83.0, 84.0, 85.0, 86.0, 87.0, 88.0, 89.0, 90.0, 91.0, 92.0, 93.0, 94.0, 95.0,
    96.0, 97.0, 97.0, 97.0,
];
static REF_KNOTS: &[f64; 102] = &KNOTS;
static REF_ELEMENTS: &[f64; 100] = &ELEMENTS;
const DEG: usize = 3;
const SPACE: usize = DEG + 1;

fn sampling(c: &mut Criterion) {
    let sample_size = 200;
    let dynamic_elements: Vec<f64> = ELEMENTS.into();
    let mut dynamic_legacy = vec![0.0];
    dynamic_legacy.extend(KNOTS);
    dynamic_legacy.push(97.0);
    let dynamic_knots: Vec<f64> = KNOTS.into();
    let dynamic_bspline = BSpline::builder()
        .elements(dynamic_elements.clone())
        .knots(dynamic_knots)
        .dynamic()
        .build()
        .unwrap();
    let const_bspline = BSpline::builder()
        .elements(ELEMENTS)
        .knots(KNOTS)
        .constant::<SPACE>()
        .build()
        .unwrap();
    let uniform_bspline = BSpline::builder()
        .elements(ELEMENTS)
        .equidistant::<f64>()
        .degree(DEG)
        .normalized()
        .constant::<SPACE>()
        .build()
        .unwrap();
    let shared_bspline = BSpline::builder()
        .elements(black_box(REF_ELEMENTS))
        .knots(black_box(REF_KNOTS))
        .constant::<SPACE>()
        .build()
        .unwrap();
    c.bench_function("sampling_dynamic_bspline", |b| {
        b.iter::<Vec<f64>, _>(|| {
            dynamic_bspline
                .by_ref()
                .take(black_box(sample_size))
                .collect()
        });
    });
    c.bench_function("sampling_const_bspline", |b| {
        b.iter::<Vec<f64>, _>(|| {
            const_bspline
                .by_ref()
                .take(black_box(sample_size))
                .collect()
        });
    });
    c.bench_function("sampling_uniform_bspline", |b| {
        b.iter::<Vec<f64>, _>(|| {
            uniform_bspline
                .by_ref()
                .take(black_box(sample_size))
                .collect()
        });
    });
    c.bench_function("sampling_shared_bspline", |b| {
        b.iter::<Vec<f64>, _>(|| {
            shared_bspline
                .by_ref()
                .take(black_box(sample_size))
                .collect()
        });
    });
}

fn creation(c: &mut Criterion) {
    let dynamic_elements: Vec<f64> = ELEMENTS.into();
    let mut dynamic_legacy = vec![0.0];
    dynamic_legacy.extend(KNOTS);
    dynamic_legacy.push(97.0);
    let dynamic_knots: Vec<f64> = KNOTS.into();
    //TODO: test shared bspline
    // bspline from the crate bspline
    c.bench_function("creation_dynamic_bspline", |b| {
        b.iter::<_, _>(|| {
            let dynamic_elements = black_box(dynamic_elements.clone());
            let dynamic_knots = black_box(dynamic_knots.clone());
            BSpline::builder()
                .elements(dynamic_elements)
                .knots(dynamic_knots)
                .dynamic()
                .build()
                .unwrap()
        });
    });
    c.bench_function("creation_const_bspline", |b| {
        b.iter::<_, _>(|| {
            BSpline::builder()
                .elements(ELEMENTS)
                .knots(KNOTS)
                .constant::<SPACE>()
                .build()
                .unwrap()
        });
    });
    c.bench_function("creation_shared_bspline", |b| {
        b.iter::<_, _>(|| {
            BSpline::builder()
                .elements(black_box(REF_ELEMENTS))
                .knots(black_box(REF_KNOTS))
                .constant::<SPACE>()
                .build()
                .unwrap()
        });
    });
    c.bench_function("creation_uniform_bspline", |b| {
        b.iter::<_, _>(|| {
            BSpline::builder()
                .elements(ELEMENTS)
                .equidistant::<f64>()
                .degree(DEG)
                .normalized()
                .constant::<SPACE>()
                .build()
                .unwrap()
        });
    });
}

criterion_group!(benches, sampling, creation);
criterion_main!(benches);
