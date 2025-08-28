use std::{f64::consts::PI, ops::{Add, Sub}};

use math::Vec2i;
use serde::{Deserialize, Serialize};


#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) struct Size2D(pub(crate) usize, pub(crate) usize);

impl Size2D {
    pub(crate) fn x(&self) -> usize {
        self.0
    }
    pub(crate) fn y(&self) -> usize {
        self.1
    }
    pub(crate) fn area(&self) -> usize {
        return self.x() * self.y()
    }
    pub(crate) fn in_bounds(&self, xy: Coord2) -> bool {
        return xy.x > 0 && xy.y > 0 && xy.x < self.0 as i32 && xy.y < self.1 as i32
    }
}

pub(crate) struct Vector2 {
    pub(crate) angle: f32,
    pub(crate) magnitude: f32
}

impl Vector2 {
    pub(crate) fn new(angle: f32, magnitude: f32) -> Vector2 {
        Vector2 { angle, magnitude }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Vec2 {
    pub(crate) x: f32,
    pub(crate) y: f32
}

impl Vec2 {

    pub(crate) fn xy(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }

    pub(crate) fn normalize(&self, mag: f32) -> Vec2 {
        let factor = self.magnitude() / mag;
        Vec2::xy(self.x / factor, self.y / factor)
    }

    pub(crate) fn magnitude(&self) -> f32 {
        let x = 0. - self.x;
        let y = 0. - self.y;
        (x * x + y * y).sqrt()
    }

}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Coord2 {
    pub(crate) x: i32,
    pub(crate) y: i32
}

impl Coord2 {

    pub(crate) fn xy(x: i32, y: i32) -> Coord2 {
        Coord2 { x, y }
    }

    pub(crate) fn dist(&self, another: &Coord2) -> f32 {
        return self.dist_squared(another).sqrt();
    }

    pub(crate) fn dist_squared(&self, another: &Coord2) -> f32 {
        let x = another.x as f32 - self.x as f32;
        let y = another.y as f32 - self.y as f32;
        return x*x + y*y
    }

    /// Returns the angle, in degrees, between this coord and the other. 0 is right
    pub(crate) fn angle_between_deg(&self, another: &Coord2) -> f64 {
        return ((f64::atan2((another.y - self.y) as f64, (another.x - self.x) as f64) * 180. / PI) + 360.) % 360.;
    }

    pub(crate) fn to_vec2(&self) -> Vec2 {
        return Vec2::xy(self.x as f32, self.y as f32)
    }

    pub(crate) fn to_vec2i(&self) -> Vec2i {
        return Vec2i(self.x, self.y)
    }

}

impl From<Vec2i> for Coord2 {
    fn from(value: Vec2i) -> Self {
        return Coord2::xy(value.0, value.1)
    }
}

impl Add for Coord2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Coord2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}