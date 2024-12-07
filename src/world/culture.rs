use std::collections::HashMap;

use crate::commons::{history_vec::Id, markovchains::MarkovChainSingleWordModel};

#[derive(Clone)]
pub struct Culture {
    pub id: Id,
    pub language: LanguagePrefab,
    pub first_name_male_model: MarkovChainSingleWordModel,
    pub first_name_female_model: MarkovChainSingleWordModel,
    pub last_name_model: MarkovChainSingleWordModel,
}

#[derive(Clone)]
pub struct LanguagePrefab {
    pub dictionary: HashMap<String, String>
}