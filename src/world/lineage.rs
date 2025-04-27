use crate::{commons::{id_vec::IdVec, rng::Rng}, Culture};

use super::culture::CultureId;


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct LineageId(usize);
impl crate::commons::id_vec::Id for LineageId {
    fn new(id: usize) -> Self {
        LineageId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Lineages = IdVec<Lineage>;

#[derive(Clone)]
pub(crate) struct Lineage {
    pub(crate) name: String,
    pub(crate) culture: CultureId,
}

impl Lineage {
    pub(crate) fn new(culture_id: CultureId, culture: &Culture) -> Self {
        let name = culture.last_name_model.generate(&Rng::rand(), 8, 15);
        Self {
            name,
            culture: culture_id
        }
    }
}