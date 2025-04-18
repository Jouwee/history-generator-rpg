use std::{cmp::Ordering, collections::HashMap};

use crate::{commons::{history_vec::Id, rng::Rng}, engine::geometry::Coord2};

use super::{faction::FactionRelation, species::SpeciesId, world::ArtifactId};


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
    pub species: SpeciesId,
    pub position: Coord2,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birth_last_name: Option<String>,
    pub importance: Importance,
    pub birth: u32,
    sex: PersonSex,
    pub death: u32,
    pub next_of_kin: Vec<NextOfKin>,
    pub civ: Option<CivilizedComponent>,
    pub possesions: Vec<ArtifactId>,
    pub appearance_hints: HashMap<String, String>
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
    pub fn new(id: Id, species: &SpeciesId, importance: Importance, birth: u32, position: Coord2) -> Person {
        let mut sex = PersonSex::Male;
        if Rng::seeded(id).rand_chance(0.5) {
            sex = PersonSex::Female;
        }
        let mut person = Person {
            id,
            species: species.clone(),
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
            possesions: Vec::new(),
            appearance_hints: HashMap::new()
        };
        person.update_appearance_hints();
        return person
    }

    pub fn set_sex(&mut self, sex: PersonSex) {
        self.sex = sex;
        self.update_appearance_hints();
    }

    pub fn sex(&self) -> &PersonSex {
        return &self.sex;
    }

    fn update_appearance_hints(&mut self) {
        match self.sex {
            PersonSex::Male => self.appearance_hints.insert(String::from("base"), String::from("male_light")),
            PersonSex::Female => self.appearance_hints.insert(String::from("base"), String::from("female_light"))
        };
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

    pub fn at_least(&self, at_least: &Importance) -> Importance {
        if self.to_numeric() > at_least.to_numeric() {
            self.clone()
        } else {
            at_least.clone()
        }
    }

    fn to_numeric(&self) -> u8 {
        match self {
            Importance::Important => 2,
            Importance::Unimportant => 1,
            Importance::Unknown => 0,
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