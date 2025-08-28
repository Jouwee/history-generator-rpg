use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

pub mod rng;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Vec2i(pub i32, pub i32);

impl Vec2i {

    pub fn x(&self) -> i32 {
        return self.0;
    }

    pub fn y(&self) -> i32 {
        return self.1;
    }

    pub fn dist(&self, another: &Vec2i) -> f32 {
        return self.dist_squared(another).sqrt();
    }

    pub fn dist_squared(&self, another: &Vec2i) -> f32 {
        let x = another.0 as f32 - self.0 as f32;
        let y = another.1 as f32 - self.1 as f32;
        return x*x + y*y
    }

}

impl Add for Vec2i {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(
            self.0 + other.0,
            self.1 + other.1,
        )
    }
}

impl Sub for Vec2i {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(
            self.0 - other.0,
            self.1 - other.1,
        )
    }
}