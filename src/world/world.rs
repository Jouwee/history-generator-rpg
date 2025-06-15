use std::{fs::File, io::Write};

use crate::{commons::id_vec::Id, game::codex::Codex, world::item::ItemId, Event, Item, Resources};

use super::{creature::{CreatureId, Creatures}, date::WorldDate, lineage::Lineages, topology::WorldTopology, unit::Units};

use crate::commons::id_vec::IdVec;

pub(crate) struct World {
    pub(crate) date: WorldDate,
    pub(crate) map: WorldTopology,
    pub(crate) units: Units,
    pub(crate) lineages: Lineages,
    pub(crate) creatures: Creatures,
    pub(crate) events: Vec<Event>,
    pub(crate) artifacts: IdVec<Item>,
    pub(crate) codex: Codex,

}

impl World {

    pub(crate) fn new(map: WorldTopology) -> World {
        return World {
            date: WorldDate::new(1, 1, 1),
            map,
            units: Units::new(),
            creatures: Creatures::new(),
            lineages: Lineages::new(),
            artifacts: IdVec::new(),
            events: Vec::new(),
            codex: Codex::new(),
        }
    }

    pub(crate) fn find_goal(&mut self, resources: &mut Resources) {
        let mut artifact = None;
        for (id, item) in self.artifacts.iter_id_val::<ItemId>() {
            let i_item = item.borrow();

            // TODO(NJ5nTVIV): Select ownerless
            // TODO(NJ5nTVIV): Older = cooler

            let mut score = 1.;
            if let Some(quality) = &i_item.quality {
                score = score * quality.quality.main_stat_multiplier();
            }
            score = score * i_item.damage_mult();

            match artifact {
                None => artifact = Some((id, item, score)),
                Some((_id, _item, c_score)) => {
                    if score > c_score {
                        artifact = Some((id, item, score));
                    }
                }
            }
        }
        if let Some((id, item, _score)) = artifact {
            // TODO(NJ5nTVIV): Title screen
            println!("You have heard of the legends of the ancient artifact {}. You set out into the world to find it's secrets.", item.borrow().name(&resources.materials));
            self.codex.artifact_mut(&id).add_name();
        }
    }

    pub(crate) fn dump_events(&self, filename: &str, resources: &Resources) {
        let mut f = File::create(filename).unwrap();
        println!("{:?} events", self.events.len());
        for event in self.events.iter() {
            match event {
                Event::CreatureBirth { date, creature_id } => {
                    let creature = self.creatures.get(creature_id);
                    let name = self.creature_desc(creature_id, date, resources);
                    let father = self.creature_desc(&creature.father, date, resources);
                    let mother = self.creature_desc(&creature.mother, date, resources);
                    writeln!(&mut f, "{}, {} was born. Father: {:?}, Mother: {:?}", self.date_desc(date), name, father, mother).unwrap();
                },
                Event::CreatureDeath { date, creature_id, cause_of_death } => {
                    let name = self.creature_desc(creature_id, date, resources);
                    writeln!(&mut f, "{}, {} died of {:?}", self.date_desc(date), name, cause_of_death).unwrap();
                },
                Event::CreatureMarriage { date, creature_id, spouse_id } => {
                    let name_a = self.creature_desc(creature_id, date, resources);
                    let name_b = self.creature_desc(spouse_id, date, resources);
                    writeln!(&mut f, "{}, {} and {} married", self.date_desc(date), name_a, name_b).unwrap();
                },
                Event::CreatureProfessionChange { date, creature_id, new_profession } => {
                    let name = self.creature_desc(creature_id, date, resources);
                    writeln!(&mut f, "{}, {} became a {:?}", self.date_desc(date), name, new_profession).unwrap();
                },
                Event::ArtifactCreated { date, artifact, creator } => {
                    let name = self.creature_desc(creator, date, resources);
                    let artifact = self.artifacts.get(artifact);
                    writeln!(&mut f, "{}, {} created {:?}", self.date_desc(date), name, artifact.name(&resources.materials)).unwrap();
                },
                Event::BurriedWithPosessions { date, creature_id } => {
                    let name = self.creature_desc(creature_id, date, resources);
                    writeln!(&mut f, "{}, {} was buried with their possessions", self.date_desc(date), name).unwrap();
                },
                Event::InheritedArtifact { date, creature_id, from, item } => {
                    let name = self.creature_desc(creature_id, date, resources);
                    let name_b = self.creature_desc(from, date, resources);
                    let artifact = self.artifacts.get(item);
                    writeln!(&mut f, "{}, {} inherited {} from {:?}", self.date_desc(date), name, artifact.name(&resources.materials), name_b).unwrap();
                },
                Event::ArtifactComission { date, creature_id, creator_id, item_id } => {
                    let name = self.creature_desc(creature_id, date, resources);
                    let name_b = self.creature_desc(creator_id, date, resources);
                    let artifact = self.artifacts.get(item_id);
                    let creature = self.creatures.get(creature_id);
                    let age = (*date - creature.birth).year();
                    writeln!(&mut f, "{}, {} commissioned {} from {:?} for his {}th birthday", self.date_desc(date), name, artifact.name(&resources.materials), name_b, age).unwrap();
                },
                Event::NewLeaderElected { date, unit_id, creature_id } => {
                    let name = self.creature_desc(creature_id, date, resources);
                    writeln!(&mut f, "{}, {} was elected new leader of {:?}", self.date_desc(date), name, *unit_id).unwrap();
                },
                Event::JoinBanditCamp { date, creature_id, unit_id, new_unit_id } => {
                    let name = self.creature_desc(creature_id, date, resources);
                    writeln!(&mut f, "{}, {} left {:?} and joined the bandits at {:?}", self.date_desc(date), name, *unit_id, *new_unit_id).unwrap();
                },
                Event::CreateBanditCamp { date, creature_id, unit_id, new_unit_id } => {
                    let name = self.creature_desc(creature_id, date, resources);
                    writeln!(&mut f, "{}, {} left {:?} and started a bandit camp at {:?}", self.date_desc(date), name, *unit_id, *new_unit_id).unwrap();
                },
            }
            
        }
    }

    fn creature_desc(&self, creature_id: &CreatureId, date: &WorldDate, resources: &Resources) -> String {
        let creature = self.creatures.get(creature_id);
        let age = (*date - creature.birth).year();
        let mut gender = "M";
        if creature.gender.is_female() {
            gender = "F";
        }
        return String::from(format!("{} [{}, {} {}]", creature.name(creature_id, &self, resources), creature_id.as_usize(), age, gender))
    }


    fn date_desc(&self, date: &WorldDate) -> String {
        return String::from(format!("{}-{}-{}", date.year(), date.month(), date.day()))
    }

}