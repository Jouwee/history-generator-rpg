use std::cmp::Ordering;

use crate::{commons::history_vec::Id, engine::Point2D};

use super::faction::FactionRelation;


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
    pub position: Point2D,
    pub first_name: String,
    pub last_name: String,
    pub birth_last_name: String,
    pub importance: Importance,
    pub birth: u32,
    pub sex: PersonSex,
    pub death: u32,
    pub culture_id: Id,
    pub next_of_kin: Vec<NextOfKin>,
    pub faction_id: Id,
    pub faction_relation: FactionRelation,
    pub leader_of_settlement: Option<Id>
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

impl Person {

    pub fn birth_name(&self) -> String {
        return format!("{} {}", self.first_name, self.birth_last_name)
    }

    pub fn name(&self) -> String {
        let title = "Commoner";
        return format!("{} {} ({:?}, {})", self.first_name, self.last_name, self.importance, title)
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

    pub fn fertility(&self, year: u32) -> f32 {
        let age = (year - self.birth) as f32;
        // https://thefertilityshop.co.uk/wp-content/uploads/2021/12/bfs-monthly-fertility-by-age-1024x569.png
        if self.sex == PersonSex::Male {
            return f32::max(0.0, -(age / 60.0).powf(2.0) + 1.0)
        } else {
            return f32::max(0.0, -(age / 40.0).powf(6.0) + 1.0)
        }
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