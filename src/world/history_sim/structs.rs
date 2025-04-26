// TODO: Break into files

use std::{cell::{Ref, RefCell, RefMut}, collections::HashMap, ops::Add};

use crate::{commons::{history_vec::Id as HId, id_vec::{Id, IdVec}}, engine::geometry::Coord2, world::{creature::{CauseOfDeath, Creature, CreatureId, Creatures, Profession}, date::WorldDate, history_generator::WorldGenerationParameters, item::Item, map_features::WorldMapFeatures, region::Region, species::SpeciesId, topology::WorldTopology, world::ArtifactId}};

pub struct World {
    pub generation_params: WorldGenerationParameters,
    pub map: WorldTopology,
    pub map_features: WorldMapFeatures,
    pub units: Vec<RefCell<Unit>>,
    pub creatures: Creatures,
    pub events: Vec<Event>,
    // pub cultures: HashMap<Id, Culture>,
    // pub factions: HistoryVec<Faction>,
    pub artifacts: IdVec<Item>,
    pub regions: HashMap<HId, Region>,

}

impl World {

    pub fn new(generation_params: WorldGenerationParameters, map: WorldTopology, regions: HashMap<HId, Region>) -> World {
        return World {
            generation_params,
            map,
            map_features: WorldMapFeatures::new(),
            units: Vec::new(),
            creatures: Creatures::new(),
            artifacts: IdVec::new(),
            events: Vec::new(),
            regions
        }
    }

    pub fn add_creature(&mut self, creature: Creature) -> CreatureId {
        self.creatures.add(creature)
    }

    pub fn add_artifact(&mut self, item: Item) -> ArtifactId {
        return self.artifacts.add(item);
    }

    pub fn get_creature(&self, id: &CreatureId) -> Ref<Creature> {
        self.creatures.get(id)
    }

    pub fn get_creature_mut(&self, id: &CreatureId) -> RefMut<Creature> {
        self.creatures.get_mut(id)
    }

}

// ----------------------

pub enum Event {
    CreatureDeath { date: WorldDate, creature_id: CreatureId, cause_of_death: CauseOfDeath },
    CreatureBirth { date: WorldDate, creature_id: CreatureId },
    CreatureMarriage { date: WorldDate, creature_id: CreatureId, spouse_id: CreatureId },
    CreatureProfessionChange { date: WorldDate, creature_id: CreatureId, new_profession: Profession },
    ArtifactCreated { date: WorldDate, artifact: ArtifactId, creator: CreatureId },
    InheritedArtifact { date: WorldDate, creature_id: CreatureId, from: CreatureId, item: ArtifactId },
    BurriedWithPosessions { date: WorldDate, creature_id: CreatureId },
    ArtifactComission { date: WorldDate, creature_id: CreatureId, creator_id: CreatureId, item_id: ArtifactId },
    NewLeaderElected { date: WorldDate, unit_id: usize, creature_id: CreatureId },
}

// ------------------

pub struct Unit {
    pub xy: Coord2,
    pub creatures: Vec<CreatureId>,
    pub cemetery: Vec<CreatureId>,
    pub unit_type: UnitType,
    pub resources: UnitResources,
    pub leader: Option<CreatureId>,
    pub artifacts: Vec<ArtifactId>
}

pub enum UnitType {
    City,
}

// -----------------

#[derive(Clone, Copy)]
pub struct UnitResources {
    // 1 unit = enough food for 1 adult for 1 year
    pub food: f32,
}

impl Add for UnitResources {
    type Output = UnitResources;

    fn add(self, other: UnitResources) -> UnitResources {
        return UnitResources {
            food: self.food + other.food
        }
    }
}

// -------------


// -----------------


// ----------------

pub struct Demographics {
    total: u16,
    children_male: u16,
    children_female: u16,
    adult_male: u16,
    adult_female: u16,
    adult_singles: u16,
    adult_married: u16,
    employed: u16,
    army: u16,
    peasants: u16,
    farmers: u16,
    artisans: u16,
    politicians: u16,
}

impl Demographics {

    pub fn new() -> Demographics {
        return Demographics {
            total: 0,
            children_male: 0,
            children_female: 0,
            adult_male: 0,
            adult_female: 0,
            adult_singles: 0,
            adult_married: 0, 
            employed: 0,
            army: 0,
            peasants: 0,
            farmers: 0,
            artisans: 0,
            politicians: 0,
        }
    }

    pub fn count(&mut self, reference: &WorldDate, creature: &Creature) {
        let age = (*reference - creature.birth).year();
        self.total += 1;
        if age < 18 {
            if creature.gender.is_male() {
                self.children_male += 1;
            } else {
                self.children_female += 1;
            }
        } else {
            if creature.gender.is_male() {
                self.adult_male += 1;
            } else {
                self.adult_female += 1;
            }
            if creature.spouse.is_none() {
                self.adult_singles += 1;
            } else {
                self.adult_married += 1;
            }
        }
        self.employed += 1;
        match creature.profession {
            Profession::None => self.employed -= 1,
            Profession::Peasant => self.peasants += 1,
            Profession::Farmer => self.farmers += 1,
            Profession::Blacksmith | Profession::Sculptor => self.artisans += 1,
            Profession::Guard => self.army += 1,
            Profession::Ruler => self.politicians += 1,
        }
    }

    pub fn print_console(&self) {
        println!("total: {}", self.total);
        println!("children_male: {} ({:.2?}%)", self.children_male, Self::pct(self.total, self.children_male));
        println!("children_female: {} ({:.2?}%)", self.children_female, Self::pct(self.total, self.children_female));
        println!("adult_male: {} ({:.2?}%)", self.adult_male, Self::pct(self.total, self.adult_male));
        println!("adult_female: {} ({:.2?}%)", self.adult_female, Self::pct(self.total, self.adult_female));
        println!("adult_singles: {} ({:.2?}%)", self.adult_singles, Self::pct(self.total, self.adult_singles));
        println!("adult_married: {} ({:.2?}%)", self.adult_married, Self::pct(self.total, self.adult_married));
        println!("employed: {} ({:.2?}%)", self.employed, Self::pct(self.total, self.employed));
        println!("army: {} ({:.2?}%)", self.army, Self::pct(self.employed, self.army));
        println!("peasants: {} ({:.2?}%)", self.peasants, Self::pct(self.employed, self.peasants));
        println!("farmers: {} ({:.2?}%)", self.farmers, Self::pct(self.employed, self.farmers));
        println!("artisans: {} ({:.2?}%)", self.artisans, Self::pct(self.employed, self.artisans));
        println!("politicians: {} ({:.2?}%)", self.politicians, Self::pct(self.employed, self.politicians));
    }

    fn pct(total: u16, count: u16) -> f32 {
        return (count as f32 / total as f32) * 100.
    }

}