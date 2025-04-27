use std::ops::{Add, Sub};


#[derive(Clone, Copy)]
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

    pub(crate) fn truncate(&self, mag: f32) -> Vec2 {
        if self.magnitude() > mag {
            return self.normalize(mag)
        }
        return self.clone()
    }

    pub(crate) fn magnitude(&self) -> f32 {
        let x = 0. - self.x;
        let y = 0. - self.y;
        (x * x + y * y).sqrt()
    }

    pub(crate) fn dist_squared(&self, another: &Vec2) -> f32 {
        let x = another.x - self.x;
        let y = another.y - self.y;
        return x*x + y*y
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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

    pub(crate) fn to_vec2(&self) -> Vec2 {
        return Vec2::xy(self.x as f32, self.y as f32)
    }

    pub(crate) fn neighbours_circle(&self, size: Size2D, r: i32) -> Vec<Coord2> {
        let mut ret = Vec::new();
        for x in (self.x - r)..(self.x + r) {
            for y in (self.y - r)..(self.y + r) {
                let coord = Coord2::xy(x, y);
                if size.in_bounds(coord) && coord.dist(self) < r as f32 {
                    ret.push(coord);
                }
            }
        }
        return ret;
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