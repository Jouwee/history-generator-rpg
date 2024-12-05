use std::ops::{Add, Sub};


#[derive(Clone, Copy)]
pub struct Size2D(pub usize, pub usize);

impl Size2D {
    pub fn x(&self) -> usize {
        self.0
    }
    pub fn y(&self) -> usize {
        self.1
    }
    pub fn area(&self) -> usize {
        return self.x() * self.y()
    }
}

pub struct Vector2 {
    pub angle: f32,
    pub magnitude: f32
}

impl Vector2 {
    pub fn new(angle: f32, magnitude: f32) -> Vector2 {
        Vector2 { angle, magnitude }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32
}

impl Vec2 {

    pub fn xy(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }

    pub fn direction(&self) -> f32 {
        let y = 0. - self.x;
        let x = 0. - self.y;
        f32::atan2(y,x)
    }

    pub fn magnitude(&self) -> f32 {
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