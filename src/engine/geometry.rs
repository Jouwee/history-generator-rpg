
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