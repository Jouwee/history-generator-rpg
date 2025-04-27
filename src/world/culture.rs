use std::collections::HashMap;

use crate::commons::{history_vec::Id, id_vec::IdVec, markovchains::MarkovChainSingleWordModel};


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct CultureId(usize);
impl crate::commons::id_vec::Id for CultureId {
    fn new(id: usize) -> Self {
        CultureId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Cultures = IdVec<Culture>;

impl Cultures {

    pub(crate) fn random(&self) -> CultureId {
        // TODO:
        return CultureId(0);
    }

}

#[derive(Clone)]
pub(crate) struct Culture {
    pub(crate) id: Id,
    pub(crate) language: LanguagePrefab,
    pub(crate) first_name_male_model: MarkovChainSingleWordModel,
    pub(crate) first_name_female_model: MarkovChainSingleWordModel,
    pub(crate) last_name_model: MarkovChainSingleWordModel,
}

#[derive(Clone)]
pub(crate) struct LanguagePrefab {
    pub(crate) dictionary: HashMap<String, String>
}