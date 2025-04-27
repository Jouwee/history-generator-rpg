use crate::commons::id_vec::IdVec;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct RegionId(usize);
impl crate::commons::id_vec::Id for RegionId {
    fn new(id: usize) -> Self {
        RegionId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Regions = IdVec<Region>;

#[derive(Debug, Clone)]
pub(crate) struct Region {
    pub(crate) elevation: (i32, i32),
    pub(crate) temperature: (u8, u8),
    pub(crate) vegetation: (f32, f32),
    pub(crate) soil_fertility_range: (f32, f32),
}