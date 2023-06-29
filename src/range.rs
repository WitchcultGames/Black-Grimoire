use gameprng::prng_traits::PrngAlgorithm;
use gameprng::xorshift128plus::XorShift128Plus;

pub struct Range {
    pub min: f32,
    pub diff: f32,
}

impl Range {
    pub fn new(min: f32, max: f32) -> Range {
        Range {
            min,
            diff: max - min,
        }
    }

    pub fn get_min(&self) -> f32 {
        self.min
    }

    pub fn get_max(&self) -> f32 {
        self.min + self.diff
    }

    pub fn get_average(&self) -> f32 {
        (self.min + self.min + self.diff) * 0.5
    }

    pub fn get_random(&self, prng: &mut XorShift128Plus) -> f32 {
        self.min + (prng.random_factor() * self.diff)
    }

    pub fn lerp(&self, t: f32) -> f32 {
        let t_clamped = 0.0_f32.max(1.0_f32.min(t));
        (1.0 - t_clamped) * self.min + t_clamped * (self.min + self.diff)
    }
}
