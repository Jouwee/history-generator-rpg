use std::collections::{hash_map::Iter, HashMap, HashSet};

use crate::{commons::history_vec::Id, WorldEvent, WorldEventEnum};

pub struct KnowledgeCodex {
    creatures: HashMap<Id, CreatureKnowledge>,
    places: HashMap<Id, PlaceKnowledge>,
    artifacts: HashMap<Id, ArtifactKnowledge>,
}

impl KnowledgeCodex {

    pub fn new() -> KnowledgeCodex {
        KnowledgeCodex {
            creatures: HashMap::new(),
            places: HashMap::new(),
            artifacts: HashMap::new(),
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
        for artifact in event.event.get_artifacts() {
            let artifact = self.artifacts.entry(artifact).or_insert(ArtifactKnowledge::new());
            artifact.events.insert(id);
            artifact.facts.insert(ArtifactFact::Name);
        }
        for place in event.event.get_settlements() {
            let place = self.places.entry(place).or_insert(PlaceKnowledge::new());
            place.events.insert(id);
            place.facts.insert(PlaceFact::Name);
        }
    }

    pub fn known_creatures(&self) -> Iter<Id, CreatureKnowledge> {
        self.creatures.iter()
    }

    pub fn creature(&self, id: &Id) -> Option<&CreatureKnowledge> {
        self.creatures.get(id)
    }

    pub fn known_places(&self) -> Iter<Id, PlaceKnowledge> {
        self.places.iter()
    }

    pub fn place(&self, id: &Id) -> Option<&PlaceKnowledge> {
        self.places.get(id)
    }

    pub fn known_artifacts(&self) -> Iter<Id, ArtifactKnowledge> {
        self.artifacts.iter()
    }

    pub fn artifact(&self, id: &Id) -> Option<&ArtifactKnowledge> {
        self.artifacts.get(id)
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

pub struct PlaceKnowledge {
    pub facts: HashSet<PlaceFact>,
    pub events: HashSet<usize>
}

impl PlaceKnowledge {
    pub fn new() -> PlaceKnowledge {
        PlaceKnowledge { facts: HashSet::new(), events: HashSet::new() } 
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum PlaceFact {
    Name
}

pub struct ArtifactKnowledge {
    pub facts: HashSet<ArtifactFact>,
    pub events: HashSet<usize>
}

impl ArtifactKnowledge {
    pub fn new() -> ArtifactKnowledge {
        ArtifactKnowledge { facts: HashSet::new(), events: HashSet::new() } 
    }
}

#[derive(Hash, PartialEq, Eq)]
pub enum ArtifactFact {
    Name,
    Description
}