pub(crate) struct MatrixIndex {
    size: (usize, usize)
}

impl MatrixIndex {
    pub(crate) fn new(size: (usize, usize)) ->  MatrixIndex {
        MatrixIndex { size }
    }

    pub(crate) fn idx(&self, x: usize, y: usize) -> usize {
        return (y * self.size.0) + x;
    }

}