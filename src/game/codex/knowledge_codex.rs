use std::collections::{hash_map::Iter, HashMap, HashSet};

use crate::{commons::history_vec::Id, WorldEvent, WorldEventEnum};

pub struct KnowledgeCodex {
    creatures: HashMap<Id, CreatureKnowledge>
}

impl KnowledgeCodex {

    pub fn new() -> KnowledgeCodex {
        KnowledgeCodex {
            creatures: HashMap::new()
        }
    }

    pub fn add_creature_fact(&mut self, id: &Id, fact: CreatureFact) {
        let creature = self.creatures.entry(*id).or_insert(CreatureKnowledge::new());
        creature.facts.insert(fact);
    }

    pub fn add_event(&mut self, id: usize, event: &WorldEvent) {
        for person in event.event.get_creatures() {
            let creature = self.creatures.entry(person).or_insert(CreatureKnowledge::new());
            creature.events.insert(id);
            creature.facts.insert(CreatureFact::Name);
            if let WorldEventEnum::PersonBorn(_) = event.event {
                creature.facts.insert(CreatureFact::Birth);
            }
            if let WorldEventEnum::PersonBorn(_) = event.event {
                creature.facts.insert(CreatureFact::Death);
            }
        }
    }

    pub fn known_creatures(&self) -> Iter<Id, CreatureKnowledge> {
        self.creatures.iter()
    }

    pub fn creature(&self, id: &Id) -> Option<&CreatureKnowledge> {
        self.creatures.get(id)
    }

}

pub struct CreatureKnowledge {
    pub facts: HashSet<CreatureFact>,
    pub events: HashSet<usize>
}

impl CreatureKnowledge {
    pub fn new() -> CreatureKnowledge {
        CreatureKnowledge { facts: HashSet::new(), events: HashSet::new() } 
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum CreatureFact {
    Name,
    Birth,
    Death
}