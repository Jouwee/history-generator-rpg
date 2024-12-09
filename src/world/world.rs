use std::{cell::{Ref, RefCell, RefMut}, collections::{BTreeMap, HashMap}};

use crate::{commons::history_vec::{HistoryVec, Id}, WorldEvents};

use super::{culture::Culture, faction::Faction, person::Person, settlement::Settlement, species::Species, topology::WorldTopology};

pub struct World {
    pub map: WorldTopology,
    pub species: HashMap<Id, Species>,
    pub cultures: HashMap<Id, Culture>,
    pub factions: HistoryVec<Faction>,
    pub settlements: HistoryVec<Settlement>,
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
        return self.inner.iter().filter(|(_id, person)| person.borrow().simulatable())
    }

    pub fn len(&self) -> usize {
        return self.inner.len()
    }

}