//! First we show why the easing functin `plateau` is called that way. Afterwards we use it
//! to generate a color gradient to categorise values neatly.
use enterpolation::{easing::Plateau, linear::Linear, Curve, Generator, Merge};
use image::{ImageBuffer, Rgba};
use palette::{Hsl, IntoColor, Mix};

// As HSL does neither implement multiplication with a scalar nor the merge trait in `topology-traits` crate,
// we need to use a newtype pattern
#[derive(Debug, Copy, Clone, Default)]
struct CustomHsl(Hsl);

impl CustomHsl {
    pub fn get_raw(&self) -> [u8; 3] {
        let srgb: palette::Srgb = self.0.into_color();
        let raw: (u8, u8, u8) = srgb.into_format().into_components();
        [raw.0, raw.1, raw.2]
    }
}

impl From<Hsl> for CustomHsl {
    fn from(from: Hsl) -> Self {
        CustomHsl(from)
    }
}

// As HSL does not implement multiplication, we have to implement the Merge trait ourselves to use enterpolation.
impl Merge<f32> for CustomHsl {
    fn merge(self, other: Self, factor: f32) -> Self {
        self.0.mix(other.0, factor).into()
    }
}

fn main() {
    //lin represents the namesake
    let lin = Linear::builder()
        .elements([0.0, 2.0, 4.0, 1.0, 0.0])
        .equidistant::<f32>()
        .domain(-1.0, 1.0)
        .easing(Plateau::new(0.7))
        .build()
        .expect("As the curve is hardcoded, it should work every time.");
    //generate #16b81e
    let green: CustomHsl = Hsl::new(123.0, 0.78, 0.40).into();
    // generate #9acd32
    let limegreen: CustomHsl = Hsl::new(80.0, 0.60, 0.50).into();
    // generate #fffb00
    let yellow: CustomHsl = Hsl::new(59.0, 1.0, 0.5).into();
    // generate #ffa500
    let orange: CustomHsl = Hsl::new(39.0, 1.0, 0.5).into();
    // generate #b30000
    let red: CustomHsl = Hsl::new(0.0, 1.0, 0.35).into();
    // first create the gradient
    let gradient = Linear::builder()
        .elements([green, limegreen, yellow, orange, red])
        .equidistant::<f32>()
        .domain(-1.0, 1.0)
        .build()
        .expect("As the curve is hardcoded, it should work every time.");
    // then do the same but use as easing `Platue`
    let plateaus = Linear::builder()
        .elements([green, limegreen, yellow, orange, red])
        .equidistant::<f32>()
        .domain(-1.0, 1.0)
        .easing(Plateau::new(0.7))
        .build()
        .expect("As the curve is hardcoded, it should work every time.");
    // make an image
    let width = 1300;
    let upper_height = 60;
    let bottom_height = 180;
    let [dmin, dmax] = gradient.domain();
    let [omin, omax] = [-1.0, 5.0];
    let mut imgbuf = ImageBuffer::new(width, upper_height + bottom_height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let raw = if y <= upper_height {
            gradient
                .gen(remap(x as f32, 0.0, width as f32, dmin, dmax))
                .get_raw()
        } else {
            let graph = lin.gen(remap(x as f32, 0.0, width as f32, dmin, dmax));
            let graph = remap(graph, omin, omax, 0.0, 1.0);
            // test if pixel falls into the area of the graph
            if (graph
                - remap(
                    (y - upper_height) as f32,
                    bottom_height as f32,
                    0.0,
                    0.0,
                    1.0,
                ))
            .abs()
                < 0.01
            {
                [0, 0, 0]
            } else {
                plateaus
                    .gen(remap(x as f32, 0.0, width as f32, dmin, dmax))
                    .get_raw()
            }
        };
        *pixel = Rgba([raw[0], raw[1], raw[2], 255]);
    }
    match imgbuf.save("examples/images/plateaus.png") {
        Ok(()) => println!("see 'examples/images/plateaus.png' for the result"),
        Err(e) => println!("failed to write 'examples/images/plateaus.png': {}", e),
    }
}

// Map t in range [a, b] to range [c, d]
fn remap(t: f32, a: f32, b: f32, c: f32, d: f32) -> f32 {
    (t - a) * ((d - c) / (b - a)) + c
}
