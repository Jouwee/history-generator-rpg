use std::collections::HashMap;

use crate::commons::{history_vec::Id, markovchains::MarkovChainSingleWordModel};

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