use crate::easing::smoothstep;
use crate::{Curve, Generator};
use num_traits::real::Real;
use num_traits::FromPrimitive;

/// Plateau is an easing curve which - therefore the name - create constant plateaus if given to
/// an interpolation which works with factors for which an easing function gets applied.
#[derive(Debug, Copy, Clone)]
pub struct Plateau<R> {
    min: R,
    max: R,
}

impl<R> Plateau<R>
where
    R: Real + FromPrimitive,
{
    /// Create a new Plateau Easing Curve with the given strength. A strength of 0.0 will be the same as
    /// the identity. A strength of 1.0 will only return 0.0 or 1.0 (depending which is nearer).
    pub fn new(strength: R) -> Self {
        let halfed = strength / R::from_usize(2).expect("Could not convert 2 into a real number");
        Plateau {
            min: R::zero() + halfed,
            max: R::one() - halfed,
        }
    }
}

/// Overclamp can be imagined as a to clamp working to hard. That is, not only will be values be
/// clamped wich are outside the defined area but also some points inside the area, depending on
/// their distance and how strong the over clamping is done.
///
/// Overclamping is done in the area of [0.0,1.0]. All values below a certain threshold will be clamped to
/// 0.0. All values above a certain threshold will be clamped to 1.0. All other values are stretched
/// such that they fill out the entire [0.0,1.0] area.
fn over_clamp<R>(input: R, min: R, max: R) -> R
where
    R: Real,
{
    if input < min {
        R::zero()
    } else if input > max {
        R::one()
    } else {
        (input - min) / (max - min)
    }
}

impl<R> Generator<R> for Plateau<R>
where
    R: Real + FromPrimitive,
{
    type Output = R;
    fn gen(&self, input: R) -> R {
        smoothstep(over_clamp(input, self.min, self.max))
    }
}

impl<R> Curve<R> for Plateau<R>
where
    R: Real + FromPrimitive,
{
    fn domain(&self) -> [R; 2] {
        [R::zero(), R::one()]
    }
}
