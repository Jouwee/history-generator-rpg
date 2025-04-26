use std::{cell::{Ref, RefCell, RefMut}, collections::{BTreeMap, HashMap}};

use crate::{commons::{history_vec::{HistoryVec, Id}, id_vec::IdVec}, WorldEvents};

use super::{culture::Culture, faction::Faction, history_generator::WorldGenerationParameters, item::Item, map_features::WorldMapFeatures, region::Region, settlement::Settlement, topology::WorldTopology};


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
