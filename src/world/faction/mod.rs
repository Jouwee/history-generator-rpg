use std::collections::{BTreeMap, BTreeSet};

use crate::commons::{history_vec::Id, rng::Rng};

#[derive(Clone, Debug)]
pub struct Faction {
    pub name: String,
    pub relations: BTreeMap<Id, f32>,
    pub leader: Id,
    pub settlements: BTreeSet<Id>,
}

impl Faction {

    pub fn new(rng: &Rng, leader: Id) -> Faction {
        let mut rng = rng.derive("faction");
        let prefixes = ["Red", "Blue", "Green", "Yellow", "Axial", "Allied", "Destructive", "Peaceful"];
        let suffixes = ["Coallition", "Legion", "Peregrins", "Colonials", "Axis", "Movement"];
        let name = format!("{} {}", prefixes[rng.randu_range(0, prefixes.len())], suffixes[rng.randu_range(0, suffixes.len())]);

        return Faction { 
            name: String::from(name),
            relations: BTreeMap::new(),
            leader,
            settlements: BTreeSet::new()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FactionRelation {
    Leader,
    Member
}