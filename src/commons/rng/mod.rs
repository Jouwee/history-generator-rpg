use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone)]
pub struct Rng {
    seed: u32
}

impl Rng {
    pub fn new(seed: u32) -> Rng {
        return Rng {
            seed
        }
    }

    pub fn seeded(seed: impl Hash) -> Rng {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        return Rng {
            seed: hasher.finish() as u32
        }
    }

    pub fn seed(&self) -> u32 {
        return self.seed;
    }

    pub fn derive(&self, deriver: impl Hash) -> Rng {
        let mut hasher = DefaultHasher::new();
        deriver.hash(&mut hasher);
        return Rng::new(self.seed.wrapping_add(hasher.finish() as u32));
    }

    pub fn next(&mut self) {
        let next = self.xor_shift(self.seed) as usize;
        self.seed = next as u32;
    }

    pub fn randu_range(&mut self, start_inclusive: usize, end_exclusive: usize) -> usize {
        let next = self.xor_shift(self.seed) as usize;
        self.seed = next as u32;
        return (next % (end_exclusive - start_inclusive)) + start_inclusive
    }

    pub fn randf(&mut self) -> f32 {
        let next = self.xor_shift(self.seed);
        return next as f32 / (u32::MAX as f32)
    }

    pub fn randf_range(&mut self, start: f32, end: f32) -> f32 {
        let next = self.randf();
        return next * (end - start) + start;
    }

    pub fn rand_chance(&mut self, chance: f32) -> bool {
        assert!(chance >= 0.0 && chance <= 1.0, "Chance must be between 0 and 1, was {}", chance);
        return self.randf() < chance
    }

    pub fn xor_shift(&mut self, v: u32) -> u32 {
        let mut x: u32 = v;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        return x;
    }

}

