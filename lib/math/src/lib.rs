use std::ops::{Add, Sub};

use serde::{Deserialize, Serialize};

pub mod rng;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Vec2i(pub i32, pub i32);

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