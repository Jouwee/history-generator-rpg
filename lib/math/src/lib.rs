use std::ops::{Add, Mul, Sub};

use serde::{Deserialize, Serialize};

pub mod rng;

#[derive(Hash, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
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

impl Mul<i32> for Vec2i {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        return Self(self.0 * rhs, self.1 * rhs)
    }
}

impl From<Vec2i> for [i32; 2] {
    fn from(value: Vec2i) -> Self {
        return [value.0, value.1]
    }
}

impl From<Vec2i> for [f64; 2] {
    fn from(value: Vec2i) -> Self {
        return [value.0 as f64, value.1 as f64]
    }
}