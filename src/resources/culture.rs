use serde::{Deserialize, Serialize};

use crate::commons::{markovchains::MarkovChainSingleWordModel, resource_map::ResourceMap};

// TODO(ROO4JcDl): Should serialize the string id, not the internal id
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct CultureId(usize);
impl crate::commons::id_vec::Id for CultureId {
    fn new(id: usize) -> Self {
        CultureId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Cultures = ResourceMap<CultureId, Culture>;

impl Cultures {

    pub(crate) fn random(&self) -> CultureId {
        // TODO:
        return CultureId(0);
    }

}

#[derive(Clone)]
pub(crate) struct Culture {
    pub(crate) first_name_male_model: MarkovChainSingleWordModel,
    pub(crate) first_name_female_model: MarkovChainSingleWordModel,
    pub(crate) last_name_model: MarkovChainSingleWordModel,
    pub(crate) city_name_model: MarkovChainSingleWordModel,
}
