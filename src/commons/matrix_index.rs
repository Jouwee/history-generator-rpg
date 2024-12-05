use crate::engine::Point2D;

pub struct MatrixIndex {
    size: (usize, usize)
}

impl MatrixIndex {
    pub fn new(size: (usize, usize)) ->  MatrixIndex {
        MatrixIndex { size }
    }

    pub fn idx(&self, x: usize, y: usize) -> usize {
        return (y * self.size.0) + x;
    }

    pub fn neighbours8(&self, x: usize, y: usize) -> Vec<usize> {
        let mut vec = vec!();
        let x0 = (x as i32 - 1).max(0) as usize;
        let x1 = (x + 1).min(self.size.0 - 1);
        let y0 = (y as i32 - 1).max(0) as usize;
        let y1 = (y + 1).min(self.size.1 - 1);
        for lx in x0..x1+1 {
            for ly in y0..y1+1 {
                vec.push(self.idx(lx, ly));
            }
        }
        return vec;
    }

    pub fn p2d(&self, point: Point2D) -> usize {
        return (point.1 * self.size.0) + point.0;
    }

    pub fn to_p2d(&self, idx: usize) -> Point2D {
        let y = idx / self.size.0;
        return Point2D(idx % self.size.0, y);
    }

    pub fn p2d_neighbours8(&self, point: Point2D) -> Vec<usize> {
        return self.neighbours8(point.0, point.1);
    }
}