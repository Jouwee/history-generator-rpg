#[derive(Clone)]
pub struct AnimationTransform {
    pub translate: [f64; 2],
    current_animation: Option<Animation>,
    animation_progress: f64
}

impl AnimationTransform {

    pub fn new() -> AnimationTransform {
        AnimationTransform {
            translate: [0.; 2],
            current_animation: None,
            animation_progress: 0.
        }
    }

    pub fn update(&mut self, delta: f64) {
        if let Some(animation) = &self.current_animation {
            self.animation_progress += delta;
            self.translate = animation.get_translate(self.animation_progress);
        }
    }

    pub fn play(&mut self, animation: &Animation) {
        self.current_animation = Some(animation.clone());
        self.animation_progress = 0.;
    }

}

#[derive(Clone, Debug)]
pub struct Animation {
    translate_keyframes: Vec<KeyFrame>
}

impl Animation {

    pub fn new() -> Animation {
        return Animation {
            translate_keyframes: Vec::new()
        }
    }

    pub fn translate(mut self, duration: f64, to: [f64; 2], smoothing: Smoothing) -> Animation {
        let mut start = 0.;
        let mut from = [0.; 2];
        if let Some(last) = self.translate_keyframes.last() {
            start = last.end;
            from = last.to;
        }
        self.translate_keyframes.push(KeyFrame {
            start,
            end: start + duration,
            from,
            to,
            smoothing
        });
        return self
    }

    pub fn get_translate(&self, progress: f64) -> [f64; 2] {
        for kf in self.translate_keyframes.iter() {
            if progress >= kf.start && progress <= kf.end {
                return [
                    kf.smoothing.interpolate(progress, [kf.start, kf.end], [kf.from[0], kf.to[0]]),
                    kf.smoothing.interpolate(progress, [kf.start, kf.end], [kf.from[1], kf.to[1]])
                ]
            }
        }
        return [0., 0.];
    }

}

#[derive(Clone, Debug)]
struct KeyFrame {
    start: f64,
    end: f64,
    from: [f64; 2],
    to: [f64; 2],
    smoothing: Smoothing
}

#[derive(Clone, Debug)]
pub enum Smoothing {
    Linear,
    EaseInOut
}

impl Smoothing {

    pub fn interpolate(&self, progress: f64, time: [f64; 2], values: [f64; 2]) -> f64 {
        let normalized = (progress - time[0]) / (time[1] - time[0]);
        let dist = values[1] - values[0];
        let offset = match self {
            Self::Linear => normalized,
            Self::EaseInOut => {
                if normalized < 0.5 {
                    2. * normalized * normalized
                } else {
                    1. - (-2. * normalized + 2.).powf(2.) / 2.
                }
            }
        };
        return values[0] + (offset * dist);
    }

}