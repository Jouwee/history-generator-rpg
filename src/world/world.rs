use std::{cell::{Ref, RefMut}, collections::HashMap};

use crate::{Item, Region, WorldGenerationParameters};

use super::{creature::{Creature, CreatureId, Creatures}, history_sim::structs::Event, map_features::WorldMapFeatures, topology::WorldTopology, unit::Units};

use crate::commons::{history_vec::Id as HId, id_vec::IdVec};



// TODO:
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct ArtifactId(usize);
impl crate::commons::id_vec::Id for ArtifactId {
    fn new(id: usize) -> Self {
        ArtifactId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}


pub(crate) struct World {
    pub(crate) generation_params: WorldGenerationParameters,
    pub(crate) map: WorldTopology,
    pub(crate) map_features: WorldMapFeatures,
    pub(crate) units: Units,
    pub(crate) creatures: Creatures,
    pub(crate) events: Vec<Event>,
    // pub(crate) cultures: HashMap<Id, Culture>,
    // pub(crate) factions: HistoryVec<Faction>,
    pub(crate) artifacts: IdVec<Item>,
    pub(crate) regions: HashMap<HId, Region>,

}

impl World {

    pub(crate) fn new(generation_params: WorldGenerationParameters, map: WorldTopology, regions: HashMap<HId, Region>) -> World {
        return World {
            generation_params,
            map,
            map_features: WorldMapFeatures::new(),
            units: Units::new(),
            creatures: Creatures::new(),
            artifacts: IdVec::new(),
            events: Vec::new(),
            regions
        }
    }

    // TODO:

    pub(crate) fn add_creature(&mut self, creature: Creature) -> CreatureId {
        self.creatures.add(creature)
    }

    pub(crate) fn add_artifact(&mut self, item: Item) -> ArtifactId {
        return self.artifacts.add(item);
    }

    pub(crate) fn get_creature(&self, id: &CreatureId) -> Ref<Creature> {
        self.creatures.get(id)
    }

    pub(crate) fn get_creature_mut(&self, id: &CreatureId) -> RefMut<Creature> {
        self.creatures.get_mut(id)
    }

}