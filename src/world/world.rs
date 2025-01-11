use std::{cell::{Ref, RefCell, RefMut}, collections::{BTreeMap, HashMap}};

use crate::{commons::{history_vec::{HistoryVec, Id}, id_vec::IdVec, resource_map::ResourceMap}, WorldEvents};

use super::{culture::Culture, faction::Faction, item::Item, material::Material, person::Person, region::Region, settlement::Settlement, species::Species, topology::WorldTopology};


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct ArtifactId(usize);
impl crate::commons::id_vec::Id for ArtifactId {
    fn new(id: usize) -> Self {
        ArtifactId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct SpeciesId(usize);
impl crate::commons::id_vec::Id for SpeciesId {
    fn new(id: usize) -> Self {
        SpeciesId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub struct World {
    pub map: WorldTopology,
    pub species: ResourceMap<SpeciesId, Species>,
    pub cultures: HashMap<Id, Culture>,
    pub regions: HashMap<Id, Region>,
    pub materials: HashMap<Id, Material>,
    pub factions: HistoryVec<Faction>,
    pub settlements: HistoryVec<Settlement>,
    pub artifacts: IdVec<Item>,
    pub people: People,
    pub events: WorldEvents
}

pub struct People {
    inner: BTreeMap<Id, RefCell<Person>>
}

impl People {
    
    pub fn new() -> People {
        People {
            inner: BTreeMap::new()
        }
    }

    pub fn get(&self, id: &Id) -> Option<Ref<Person>> {
        let option = self.inner.get(id);
        match option {
            None => None,
            Some(ref_cell) => Some(ref_cell.borrow())
        }
    }

    pub fn get_mut(&self, id: &Id) -> Option<RefMut<Person>> {
        let option = self.inner.get(id);
        match option {
            None => None,
            Some(ref_cell) => Some(ref_cell.borrow_mut())
        }
    }

    pub fn insert(&mut self, person: Person) {
        self.inner.insert(person.id, RefCell::new(person));
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Id, &RefCell<Person>)> {
        return self.inner.iter().filter(|(_id, person)| {
            if let Ok(person) = person.try_borrow() {
                return person.alive()
            }
            return false
        })
    }

    pub fn ids(&self) -> Vec<Id> {
        // It's actually faster to borrow and test simulatability than clone the extra IDs
        return self.iter().map(|(id, _)| id.clone()).collect()
    }

    pub fn len(&self) -> usize {
        return self.inner.len()
    }

}