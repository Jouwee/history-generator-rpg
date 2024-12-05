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
}