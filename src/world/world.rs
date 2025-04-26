use std::{fs::File, io::Write, cell::{Ref, RefMut}, collections::HashMap};

use crate::{Event, Item, Region, Resources, WorldGenerationParameters};

use super::{creature::{Creature, CreatureId, Creatures}, date::WorldDate, map_features::WorldMapFeatures, topology::WorldTopology, unit::Units};

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

    pub(crate) fn dump_events(&self, filename: &str, resources: &Resources) {
        let mut f = File::create(filename).unwrap();
        println!("{:?} events", self.events.len());
        for event in self.events.iter() {
            match event {
                Event::CreatureBirth { date, creature_id } => {
                    let creature = self.get_creature(creature_id);
                    let name = self.creature_desc(creature_id, date);
                    let father = self.creature_desc(&creature.father, date);
                    let mother = self.creature_desc(&creature.mother, date);
                    writeln!(&mut f, "{}, {} was born. Father: {:?}, Mother: {:?}", self.date_desc(date), name, father, mother).unwrap();
                },
                Event::CreatureDeath { date, creature_id, cause_of_death } => {
                    let name = self.creature_desc(creature_id, date);
                    writeln!(&mut f, "{}, {} died of {:?}", self.date_desc(date), name, cause_of_death).unwrap();
                },
                Event::CreatureMarriage { date, creature_id, spouse_id } => {
                    let name_a = self.creature_desc(creature_id, date);
                    let name_b = self.creature_desc(spouse_id, date);
                    writeln!(&mut f, "{}, {} and {} married", self.date_desc(date), name_a, name_b).unwrap();
                },
                Event::CreatureProfessionChange { date, creature_id, new_profession } => {
                    let name = self.creature_desc(creature_id, date);
                    writeln!(&mut f, "{}, {} became a {:?}", self.date_desc(date), name, new_profession).unwrap();
                },
                Event::ArtifactCreated { date, artifact, creator } => {
                    let name = self.creature_desc(creator, date);
                    let artifact = self.artifacts.get(artifact);
                    writeln!(&mut f, "{}, {} created {:?}", self.date_desc(date), name, artifact.name(&resources.materials)).unwrap();
                },
                Event::BurriedWithPosessions { date, creature_id } => {
                    let name = self.creature_desc(creature_id, date);
                    writeln!(&mut f, "{}, {} was buried with their possessions", self.date_desc(date), name).unwrap();
                },
                Event::InheritedArtifact { date, creature_id, from, item } => {
                    let name = self.creature_desc(creature_id, date);
                    let name_b = self.creature_desc(from, date);
                    let artifact = self.artifacts.get(item);
                    writeln!(&mut f, "{}, {} inherited {} from {:?}", self.date_desc(date), name, artifact.name(&resources.materials), name_b).unwrap();
                },
                Event::ArtifactComission { date, creature_id, creator_id, item_id } => {
                    let name = self.creature_desc(creature_id, date);
                    let name_b = self.creature_desc(creator_id, date);
                    let artifact = self.artifacts.get(item_id);
                    let creature = self.get_creature(creature_id);
                    let age = (*date - creature.birth).year();
                    writeln!(&mut f, "{}, {} commissioned {} from {:?} for his {}th birthday", self.date_desc(date), name, artifact.name(&resources.materials), name_b, age).unwrap();
                },
                Event::NewLeaderElected { date, unit_id, creature_id } => {
                    let name = self.creature_desc(creature_id, date);
                    writeln!(&mut f, "{}, {} was elected new leader of {:?}", self.date_desc(date), name, *unit_id).unwrap();
                }
            }
            
        }
    }

    fn creature_desc(&self, creature_id: &CreatureId, date: &WorldDate) -> String {
        let creature = self.get_creature(creature_id);
        let age = (*date - creature.birth).year();
        let mut gender = "M";
        if creature.gender.is_female() {
            gender = "F";
        }
        return String::from(format!("[{:?}, {:?} {:?}]", creature_id, age, gender))
    }


    fn date_desc(&self, date: &WorldDate) -> String {
        return String::from(format!("{}-{}-{}", date.year(), date.month(), date.day()))
    }

}