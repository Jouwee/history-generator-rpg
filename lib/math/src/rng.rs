use std::hash::{DefaultHasher, Hash, Hasher};

pub struct Rng(u64);

impl Rng {

    pub fn new(seed: u64) -> Self {
        return Self(seed);
    }

    pub fn hash(&self, hashable: impl Hash) -> Rng {
        let mut hasher = DefaultHasher::new();
        hashable.hash(&mut hasher);
        return Rng::new(self.0.wrapping_add(hasher.finish()));
    }

    pub fn u64(&mut self) -> u64 {
        self.next()
    }

    pub fn u32(&mut self) -> u32 {
        self.next() as u32
    }

    pub fn f64(&mut self) -> f64 {
        self.next() as f64 / (u64::MAX as f64)
    }

    /// SplitMix64 implementation
    fn next(&mut self) -> u64 {
        // Increment the state variable
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        // Work the state to remove predictability
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        return z ^ (z >> 31);
    }

}