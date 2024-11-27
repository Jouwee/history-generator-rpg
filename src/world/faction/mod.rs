use std::collections::{BTreeSet, HashMap};

use crate::{commons::rng::Rng, engine::Id};

#[derive(Clone, Debug)]
pub struct Faction {
    pub id: Id,
    pub name: String,
    pub relations: HashMap<Id, f32>,
    pub leader: Id,
    pub settlements: BTreeSet<Id>,
}

impl Faction {

    pub fn new(rng: &Rng, id: Id, leader: Id) -> Faction {
        let mut rng = rng.derive("faction");
        let prefixes = ["Red", "Blue", "Green", "Yellow", "Axial", "Allied", "Destructive", "Peaceful"];
        let suffixes = ["Coallition", "Legion", "Peregrins", "Colonials", "Axis", "Movement"];
        let name = format!("{} {}", prefixes[rng.randu_range(0, prefixes.len())], suffixes[rng.randu_range(0, suffixes.len())]);

        return Faction { 
            id: id,
            name: String::from(name),
            relations: HashMap::new(),
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