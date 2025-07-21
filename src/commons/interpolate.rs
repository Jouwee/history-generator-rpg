use std::f64::consts::PI;

pub(crate) fn lerp(from: f64, to: f64, percentile: f64) -> f64 {
    return from + ((to - from) * percentile);
}


#[derive(Clone, Debug)]
pub(crate) enum Interpolate {
    EaseInSine,
    EaseOutSine,
}

impl Interpolate {

    pub(crate) fn interpolate(&self, x: f64) -> f64 {
        return match self {
            Self::EaseInSine => 1. - f64::cos((x * PI) / 2.),
            Self::EaseOutSine => f64::sin((x * PI) / 2.),
        }
    }

}