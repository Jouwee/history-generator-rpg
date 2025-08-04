use std::{hash::{DefaultHasher, Hash, Hasher}, time::Instant};

#[derive(Clone)]
pub(crate) struct Rng {
    seed: u32
}

impl Rng {

    pub(crate) fn rand() -> Rng {
        let now = Instant::now();
        return Rng::seeded(now);
    }
    
    pub(crate) fn new(seed: u32) -> Rng {
        let mut rng = Rng {
            seed
        };
        // This pseudo algorithm has a downside where close digits generate close results. So 0 and 1 are similar.
        // This next jumps to the next random number to avoid this downside.
        rng.next();
        rng.next();
        return rng;
    }

    pub(crate) fn seeded(seed: impl Hash) -> Rng {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        return Rng {
            seed: hasher.finish() as u32
        }
    }

    pub(crate) fn seed(&self) -> u32 {
        return self.seed;
    }

    pub(crate) fn derive(&self, deriver: impl Hash) -> Rng {
        let mut hasher = DefaultHasher::new();
        deriver.hash(&mut hasher);
        return Rng::new(self.seed.wrapping_add(hasher.finish() as u32));
    }

    pub(crate) fn next(&mut self) {
        let next = self.xor_shift(self.seed) as usize;
        self.seed = next as u32;
    }
    
    pub(crate) fn randi_range(&mut self, start_inclusive: i32, end_exclusive: i32) -> i32 {
        return self.randf_range(start_inclusive as f32, end_exclusive as f32) as i32;
    }

    pub(crate) fn randu_range(&mut self, start_inclusive: usize, end_exclusive: usize) -> usize {
        if start_inclusive == end_exclusive {
            return end_exclusive
        }
        let next = self.xor_shift(self.seed) as usize;
        self.seed = next as u32;
        return (next % (end_exclusive - start_inclusive)) + start_inclusive
    }

    pub(crate) fn randf(&mut self) -> f32 {
        let next = self.xor_shift(self.seed);
        return next as f32 / (u32::MAX as f32)
    }

    pub(crate) fn randf_range(&mut self, start: f32, end: f32) -> f32 {
        let next = self.randf();
        return next * (end - start) + start;
    }

    pub(crate) fn rand_chance(&mut self, chance: f32) -> bool {
        assert!(chance >= 0.0 && chance <= 1.0, "Chance must be between 0 and 1, was {}", chance);
        return self.randf() < chance
    }

    pub (crate) fn item<'a, U>(&mut self, array: &'a Vec<U>) -> Option<&'a U> {
        let i = self.randu_range(0, array.len());
        return array.get(i);
    }

    pub (crate) fn shuffle<U>(&mut self, mut array: Vec<U>) -> Vec<U> {
        let mut new = Vec::new();
        while array.len() > 0 {
            let item = array.remove(self.randu_range(0, array.len()));
            new.push(item);
        }
        return new;
    }

    fn xor_shift(&mut self, v: u32) -> u32 {
        let mut x: u32 = v;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        return x;
    }

}

