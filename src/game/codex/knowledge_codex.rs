use std::collections::{hash_map::Iter, HashMap, HashSet};

use crate::{commons::history_vec::Id, world::world::ArtifactId, WorldEvent, WorldEventEnum};

pub(crate) struct KnowledgeCodex {
    creatures: HashMap<Id, CreatureKnowledge>,
    places: HashMap<Id, PlaceKnowledge>,
    artifacts: HashMap<ArtifactId, ArtifactKnowledge>,
}

impl KnowledgeCodex {

    pub(crate) fn new() -> KnowledgeCodex {
        KnowledgeCodex {
            creatures: HashMap::new(),
            places: HashMap::new(),
            artifacts: HashMap::new(),
        }
    }

    pub(crate) fn add_creature_fact(&mut self, id: &Id, fact: CreatureFact) {
        let creature = self.creatures.entry(*id).or_insert(CreatureKnowledge::new());
        creature.facts.insert(fact);
    }

    pub(crate) fn add_event(&mut self, id: usize, event: &WorldEvent) {
        for creature in event.event.get_creatures() {
            let creature = self.creatures.entry(creature).or_insert(CreatureKnowledge::new());
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
        for place in event.event.get_units() {
            let place = self.places.entry(place).or_insert(PlaceKnowledge::new());
            place.events.insert(id);
            place.facts.insert(PlaceFact::Name);
        }
    }

    pub(crate) fn known_creatures(&self) -> Iter<Id, CreatureKnowledge> {
        self.creatures.iter()
    }

    pub(crate) fn creature(&self, id: &Id) -> Option<&CreatureKnowledge> {
        self.creatures.get(id)
    }

    pub(crate) fn known_places(&self) -> Iter<Id, PlaceKnowledge> {
        self.places.iter()
    }

    pub(crate) fn place(&self, id: &Id) -> Option<&PlaceKnowledge> {
        self.places.get(id)
    }

    pub(crate) fn known_artifacts(&self) -> Iter<ArtifactId, ArtifactKnowledge> {
        self.artifacts.iter()
    }

    pub(crate) fn artifact(&self, id: &ArtifactId) -> Option<&ArtifactKnowledge> {
        self.artifacts.get(id)
    }

}

pub(crate) struct CreatureKnowledge {
    pub(crate) facts: HashSet<CreatureFact>,
    pub(crate) events: HashSet<usize>
}

impl CreatureKnowledge {
    pub(crate) fn new() -> CreatureKnowledge {
        CreatureKnowledge { facts: HashSet::new(), events: HashSet::new() } 
    }
}

#[derive(Hash, PartialEq, Eq)]
pub(crate) enum CreatureFact {
    Name,
    Birth,
    Death
}

pub(crate) struct PlaceKnowledge {
    pub(crate) facts: HashSet<PlaceFact>,
    pub(crate) events: HashSet<usize>
}

impl PlaceKnowledge {
    pub(crate) fn new() -> PlaceKnowledge {
        PlaceKnowledge { facts: HashSet::new(), events: HashSet::new() } 
    }
}

#[derive(Hash, PartialEq, Eq)]
pub(crate) enum PlaceFact {
    Name
}

pub(crate) struct ArtifactKnowledge {
    pub(crate) facts: HashSet<ArtifactFact>,
    pub(crate) events: HashSet<usize>
}

impl ArtifactKnowledge {
    pub(crate) fn new() -> ArtifactKnowledge {
        ArtifactKnowledge { facts: HashSet::new(), events: HashSet::new() } 
    }
}

#[derive(Hash, PartialEq, Eq)]
pub(crate) enum ArtifactFact {
    Name,
    Description
}