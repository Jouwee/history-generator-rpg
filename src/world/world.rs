

// TODO:
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct ArtifactId(usize);
impl crate::commons::id_vec::Id for ArtifactId {
    fn new(id: usize) -> Self {
        ArtifactId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}
