use std::cmp::Ordering;

use crate::{commons::{history_vec::Id, rng::Rng}, engine::geometry::Coord2};

use super::{faction::FactionRelation, species::Species};


#[derive(Clone, PartialEq, Debug)]
pub enum PersonSex {
    Male,
    Female
}

impl PersonSex {

    pub fn opposite(&self) -> PersonSex {
        match self {
            PersonSex::Male => return PersonSex::Female,
            PersonSex::Female => return PersonSex::Male,
        }
    }

}

#[derive(Clone, Debug)]
pub struct Person {
    pub id: Id,
    pub species: Id,
    pub position: Coord2,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birth_last_name: Option<String>,
    pub importance: Importance,
    pub birth: u32,
    pub sex: PersonSex,
    pub death: u32,
    pub next_of_kin: Vec<NextOfKin>,
    pub civ: Option<CivilizedComponent>,
    pub possesions: Vec<Id>
}

#[derive(Clone, Debug)]
pub struct CivilizedComponent {
    pub culture: Id,
    pub faction: Id,
    pub faction_relation: FactionRelation,
    pub leader_of_settlement: Option<Id>
}

impl Person {

    // TODO: random Age
    pub fn new(id: Id, species: &Species, importance: Importance, birth: u32, position: Coord2) -> Person {
        let mut sex = PersonSex::Male;
        if Rng::seeded(id).rand_chance(0.5) {
            sex = PersonSex::Female;
        }
        Person {
            id,
            species: species.id,
            importance,
            position,
            birth,
            first_name: None,
            last_name: None,
            birth_last_name: None,
            sex,
            death: 0,
            next_of_kin: vec!(),
            civ: None,
            possesions: Vec::new()
        }
    }

    pub fn civilization(mut self, civilization: &Option<CivilizedComponent>) -> Self {
        match civilization {
            Some(civ) => self.civ = Some(civ.clone()),
            None => self.civ = None
        }
        self
    }

    pub fn birth_name(&self) -> Option<String> {
        if let Some(first_name) = &self.first_name {
            if let Some(birth_last_name) = &self.birth_last_name {
                return Some(format!("{} {}", first_name, birth_last_name))
            }
        }
        return None
    }

    pub fn name(&self) -> Option<String> {
        if let Some(first_name) = &self.first_name {
            if let Some(last_name) = &self.last_name {
                return Some(format!("{} {}", first_name, last_name))
            }
        }
        return None
    }

    pub fn simulatable(&self) -> bool {
        self.alive() && self.importance != Importance::Unknown
    }

    pub fn alive(&self) -> bool {
        return self.death == 0
    }

    pub fn spouse(&self) -> Option<&Id> {
        let spouse = self.next_of_kin.iter().find(|r| r.relative == Relative::Spouse);
        if let Some(spouse) = spouse {
            return Some(&spouse.person_id)
        };
        return None
    }

    pub fn find_next_of_kin(&self, relative: Relative) -> Option<&Id> {
        let spouse = self.next_of_kin.iter().find(|r| r.relative == relative);
        if let Some(spouse) = spouse {
            return Some(&spouse.person_id)
        };
        return None
    }

    pub fn sorted_heirs(&self) -> Vec<NextOfKin> {
        let priorities = [
            Relative::Child,
            Relative::Spouse,
            Relative::Sibling,
        ];
        
        let mut sorted = self.next_of_kin.clone();
        sorted.sort_by(|kin1, kin2| {
            let priority_1 = priorities.iter().position(|r| *r == kin1.relative);
            let priority_2 = priorities.iter().position(|r| *r == kin2.relative);
            if priority_1 != priority_2 {
                return priority_1.cmp(&priority_2);
            }
            return Ordering::Equal;
        });
        return sorted
    }

}


#[derive(Clone, PartialEq, Debug)]
pub enum Importance {
    Important,
    Unimportant,
    Unknown
}

impl Importance {
    pub fn lower(&self) -> Importance {
        match self {
            Importance::Important => return Importance::Unimportant,
            Importance::Unimportant => return Importance::Unknown,
            Importance::Unknown => return Importance::Unknown,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NextOfKin {
    pub person_id: Id,
    pub relative: Relative
}

#[derive(Clone, PartialEq, Debug)]
pub enum Relative {
    Spouse,
    Sibling,
    Parent,
    Child,
}