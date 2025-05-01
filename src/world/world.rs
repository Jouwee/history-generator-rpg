use std::{fs::File, io::Write};

use crate::{commons::id_vec::Id, Event, Item, Resources};

use super::{creature::{CreatureId, Creatures}, culture::Cultures, date::WorldDate, lineage::Lineages, map_features::WorldMapFeatures, topology::WorldTopology, unit::Units};

use crate::commons::id_vec::IdVec;

pub(crate) struct World {
    pub(crate) map: WorldTopology,
    pub(crate) map_features: WorldMapFeatures,
    pub(crate) units: Units,
    pub(crate) lineages: Lineages,
    pub(crate) creatures: Creatures,
    pub(crate) events: Vec<Event>,
    pub(crate) cultures: Cultures,
    pub(crate) artifacts: IdVec<Item>,

}

impl World {

    pub(crate) fn new(map: WorldTopology, cultures: Cultures) -> World {
        return World {
            map,
            map_features: WorldMapFeatures::new(),
            units: Units::new(),
            creatures: Creatures::new(),
            lineages: Lineages::new(),
            artifacts: IdVec::new(),
            events: Vec::new(),
            cultures
        }
    }

    pub(crate) fn dump_events(&self, filename: &str, resources: &Resources) {
        let mut f = File::create(filename).unwrap();
        println!("{:?} events", self.events.len());
        for event in self.events.iter() {
            match event {
                Event::CreatureBirth { date, creature_id } => {
                    let creature = self.creatures.get(creature_id);
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
                    let creature = self.creatures.get(creature_id);
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
        let creature = self.creatures.get(creature_id);
        let age = (*date - creature.birth).year();
        let mut gender = "M";
        if creature.gender.is_female() {
            gender = "F";
        }
        return String::from(format!("{} [{}, {} {}]", creature.name(creature_id, &self), creature_id.as_usize(), age, gender))
    }


    fn date_desc(&self, date: &WorldDate) -> String {
        return String::from(format!("{}-{}-{}", date.year(), date.month(), date.day()))
    }

}