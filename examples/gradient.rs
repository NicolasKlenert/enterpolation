use enterpolation::{bspline::BSpline, Curve, Generator, Merge};
use image::{ImageBuffer, Rgba};
use palette::{Hsl, IntoColor, Mix};

// As HSL does neither implement multiplication with a scalar nor the merge trait in `topology-traits` crate,
// we need to use a newtype pattern
#[derive(Debug, Copy, Clone, Default)]
struct CustomHsl(Hsl);

impl From<Hsl> for CustomHsl {
    fn from(from: Hsl) -> Self {
        CustomHsl(from)
    }
}

// As HSL does not implement multiplication, we have to implement the Merge trait ourself to use enterpolation.
impl Merge<f32> for CustomHsl {
    fn merge(self, other: Self, factor: f32) -> Self {
        self.0.mix(other.0, factor).into()
    }
}

fn main() {
    //generate #1f005c
    let navy: CustomHsl = Hsl::new(260.0, 1.0, 0.18).into();
    // generate #8c00a0
    let magenta: CustomHsl = Hsl::new(292.0, 1.0, 0.314).into();
    // generate #e30084
    let pink: CustomHsl = Hsl::new(325.0, 1.0, 0.445).into();
    // generate #ff2830
    let red: CustomHsl = Hsl::new(358.0, 1.0, 0.578).into();
    // generate #ffb56b
    let sandy: CustomHsl = Hsl::new(30.0, 1.0, 0.71).into();
    // we want to use a bspline with degree 3
    let spline = BSpline::builder()
        .clamped()
        .elements([navy, magenta, pink, red, sandy])
        .equidistant::<f32>()
        .degree(3)
        .normalized()
        .constant::<4>()
        .build()
        .expect("As the curve is hardcoded, this should always work");
    // make an image from the gradient
    let width = 1300;
    let height = 60;
    let [dmin, dmax] = spline.domain();
    let mut imgbuf = ImageBuffer::new(width, height);

    for (x, _, pixel) in imgbuf.enumerate_pixels_mut() {
        let hsl = spline.gen(remap(x as f32, 0.0, width as f32, dmin, dmax)).0;
        let srgb: palette::Srgb = hsl.into_color();
        let raw: (u8, u8, u8) = srgb.into_format().into_components();
        *pixel = Rgba([raw.0, raw.1, raw.2, 255]);
    }
    match imgbuf.save("examples/images/gradient.png") {
        Ok(()) => println!("see 'examples/images/gradient.png' for the result"),
        Err(e) => println!("failed to write 'examples/images/gradient.png': {}", e),
    }
}

// Map t in range [a, b] to range [c, d]
fn remap(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    (t - a) * ((d - c) / (b - a)) + c
}
