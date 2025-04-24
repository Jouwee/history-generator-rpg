// TODO: Break into files

use std::{cell::{Ref, RefCell, RefMut}, ops::Add};

use crate::{commons::id_vec::Id, engine::geometry::Coord2, world::species::SpeciesId};

#[derive(Clone)]
pub struct WorldDate {
    pub year: i32,
}

impl WorldDate {

    pub fn year(year: i32) -> WorldDate {
        WorldDate { year }
    }

    pub fn add(&self, another: &WorldDate) -> WorldDate {
        return WorldDate {
            year: self.year + another.year
        }
    }

    pub fn subtract(&self, another: &WorldDate) -> WorldDate {
        return WorldDate {
            year: self.year - another.year
        }
    }

}

// ----------------------

pub struct World {
    pub units: Vec<RefCell<Unit>>,
    pub creatures: Vec<RefCell<Creature>>,
    pub events: Vec<Event>
}

impl World {

    pub fn new() -> World {
        return World {
            units: Vec::new(),
            creatures: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn add_creature(&mut self, creature: Creature) -> CreatureId {
        self.creatures.push(RefCell::new(creature));
        return CreatureId(self.creatures.len() - 1)
    }

    pub fn get_creature(&self, id: &CreatureId) -> Ref<Creature> {
        self.creatures.get(id.as_usize()).expect("Invalid ID").borrow()
    }

    pub fn get_creature_mut(&self, id: &CreatureId) -> RefMut<Creature> {
        self.creatures.get(id.as_usize()).expect("Invalid ID").borrow_mut()
    }

}

// ----------------------

pub enum Event {
    CreatureDeath { date: WorldDate, creature_id: CreatureId, cause_of_death: CauseOfDeath },
    CreatureBirth { date: WorldDate, creature_id: CreatureId },
    CreatureMarriage { date: WorldDate, creature_id: CreatureId, spouse_id: CreatureId },
    CreatureProfessionChange { date: WorldDate, creature_id: CreatureId, new_profession: Profession },
}

// ------------------

pub struct Unit {
    pub xy: Coord2,
    pub creatures: Vec<CreatureId>,
    pub unit_type: UnitType,
    pub resources: UnitResources
}

pub enum UnitType {
    City,
}

// --------------------------

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct CreatureId(usize);
impl CreatureId {
    pub fn ancients() -> CreatureId {
        return CreatureId(0);
    }
}
impl crate::commons::id_vec::Id for CreatureId {
    fn new(id: usize) -> Self {
        CreatureId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub struct Creature {
    pub species: SpeciesId,
    pub birth: WorldDate,
    pub gender: CreatureGender,
    pub death: Option<(WorldDate, CauseOfDeath)>,
    pub profession: Profession,
    pub father: CreatureId,
    pub mother: CreatureId,
    pub spouse: Option<CreatureId>,
    pub offspring: Vec<CreatureId>,
    pub details: Option<CreatureDetails>
}

pub struct CreatureDetails {

}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CreatureGender {
    Male, Female
}

impl CreatureGender {

    pub fn is_male(&self) -> bool {
        if let CreatureGender::Male = self {
            return true
        }
        return false
    }

    pub fn is_female(&self) -> bool {
        if let CreatureGender::Female = self {
            return true
        }
        return false
    }

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

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Profession {
    // Someone that doesn't work. Usually children and elders, but could be reserved for nitwits.
    None,
    // Workers
    // A peasant is someone trying to make the ends meet. Usually poor, they produce enough food to feed themselves and maybe a child, and pay a little in taxes.
    Peasant,
    Farmer,
    // Military
    Guard,
    // Artisans
    Blacksmith,
    // Political
    Ruler
}

impl Profession {

    pub fn base_resource_production(&self) -> UnitResources {
        match self {
            Profession::None => UnitResources { food: 0. },
            Profession::Peasant => UnitResources { food: 1.5 },
            Profession::Farmer => UnitResources { food: 3.0 },
            Profession::Guard => UnitResources { food: 0. },
            Profession::Blacksmith => UnitResources { food: 0. },
            Profession::Ruler => UnitResources { food: 0. },
        }
    }

    pub fn is_for_life(&self) -> bool {
        match self {
            Profession::None | Profession::Peasant | Profession::Farmer  | Profession::Guard | Profession::Blacksmith => false,
            Profession::Ruler => true,
        }
    }

}

// -----------------

#[derive(Clone, Copy, Debug)]
pub enum CauseOfDeath {
    OldAge,
    Starvation,
    Disease,
}

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
        let age = reference.subtract(&creature.birth).year;
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
            Profession::Blacksmith => self.artisans += 1,
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