use std::cmp::Ordering;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Point2D(pub usize, pub usize);

impl Point2D {

    pub fn dist_squared(&self, another: &Point2D) -> f32 {
        let x = another.0 as f32 - self.0 as f32;
        let y = another.1 as f32 - self.1 as f32;
        return x*x + y*y
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, PartialOrd)]
pub struct Id(pub i32);

impl Ord for Id {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl Id {
    pub fn next(&mut self) -> Id {
        let clone = self.clone();
        self.0 = self.0 + 1;
        clone
    }
}
