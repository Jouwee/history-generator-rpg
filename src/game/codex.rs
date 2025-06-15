use std::collections::HashMap;

use crate::{commons::bitmask::{bitmask_get, bitmask_set}, world::{creature::CreatureId, item::ItemId}};

pub(crate) struct Codex {
    creatures: HashMap<CreatureId, CreatureCodex>,
    artifacts: HashMap<ItemId, ArtifactCodex>,
}

impl Codex {

    pub(crate) fn new() -> Self {
        Codex {
            creatures: HashMap::new(),
            artifacts: HashMap::new(),
        }
    }

    pub(crate) fn creatures(&self) -> std::collections::hash_map::Keys<'_, CreatureId, CreatureCodex> {
        return self.creatures.keys();
    }

    pub(crate) fn creature(&self, creature_id: &CreatureId) -> Option<&CreatureCodex> {
        return self.creatures.get(creature_id);
    }

    pub(crate) fn creature_mut(&mut self, creature_id: &CreatureId) -> &mut CreatureCodex {
        if !self.creatures.contains_key(creature_id) {
            self.creatures.insert(*creature_id, CreatureCodex { facts_bitmask: 0, events: Vec::new() });    
        }
        return self.creatures.get_mut(creature_id).expect("Just inserted");
    }

    pub(crate) fn artifacts(&self) -> std::collections::hash_map::Keys<'_, ItemId, ArtifactCodex> {
        return self.artifacts.keys();
    }

    pub(crate) fn artifact(&self, artifact_id: &ItemId) -> Option<&ArtifactCodex> {
        return self.artifacts.get(artifact_id);
    }

    pub(crate) fn artifact_mut(&mut self, artifact_id: &ItemId) -> &mut ArtifactCodex {
        if !self.artifacts.contains_key(artifact_id) {
            self.artifacts.insert(*artifact_id, ArtifactCodex { facts_bitmask: 0, events: Vec::new() });    
        }
        return self.artifacts.get_mut(artifact_id).expect("Just inserted");
    }

}

const CREATURE_FACT_NAME: u8 = 0b0000_0001;
const CREATURE_FACT_BIRTH: u8 = 0b0000_0010;
const CREATURE_FACT_DEATH: u8 = 0b0000_0100;
const CREATURE_FACT_APPEARANCE: u8 = 0b0000_1000;
const CREATURE_FACT_FATHER: u8 = 0b0001_0000;
const CREATURE_FACT_MOTHER: u8 = 0b0010_0000;

pub(crate) struct CreatureCodex {
    facts_bitmask: u8,
    events: Vec<usize>,
}

impl CreatureCodex {

    pub(crate) fn know_name(&self) -> bool {
        return bitmask_get(self.facts_bitmask, CREATURE_FACT_NAME);
    }

    pub(crate) fn add_name(&mut self) {
        self.facts_bitmask = bitmask_set(self.facts_bitmask, CREATURE_FACT_NAME);
    }

    pub(crate) fn know_birth(&self) -> bool {
        return bitmask_get(self.facts_bitmask, CREATURE_FACT_BIRTH);
    }

    pub(crate) fn add_birth(&mut self) {
        self.facts_bitmask = bitmask_set(self.facts_bitmask, CREATURE_FACT_BIRTH);
    }

    pub(crate) fn know_death(&self) -> bool {
        return bitmask_get(self.facts_bitmask, CREATURE_FACT_DEATH);
    }

    pub(crate) fn add_death(&mut self) {
        self.facts_bitmask = bitmask_set(self.facts_bitmask, CREATURE_FACT_DEATH);
    }

    pub(crate) fn know_appearance(&self) -> bool {
        return bitmask_get(self.facts_bitmask, CREATURE_FACT_APPEARANCE);
    }

    pub(crate) fn add_appearance(&mut self) {
        self.facts_bitmask = bitmask_set(self.facts_bitmask, CREATURE_FACT_APPEARANCE);
    }

    pub(crate) fn know_father(&self) -> bool {
        return bitmask_get(self.facts_bitmask, CREATURE_FACT_FATHER);
    }

    pub(crate) fn add_father(&mut self) {
        self.facts_bitmask = bitmask_set(self.facts_bitmask, CREATURE_FACT_FATHER);
    }

    pub(crate) fn know_mother(&self) -> bool {
        return bitmask_get(self.facts_bitmask, CREATURE_FACT_MOTHER);
    }

    pub(crate) fn add_mother(&mut self) {
        self.facts_bitmask = bitmask_set(self.facts_bitmask, CREATURE_FACT_MOTHER);
    }

}

const ARTIFACT_FACT_NAME: u8 = 0b0000_0001;

pub(crate) struct ArtifactCodex {
    facts_bitmask: u8,
    events: Vec<usize>,
}

impl ArtifactCodex {

    pub(crate) fn know_name(&self) -> bool {
        return bitmask_get(self.facts_bitmask, ARTIFACT_FACT_NAME);
    }

    pub(crate) fn add_name(&mut self) {
        self.facts_bitmask = bitmask_set(self.facts_bitmask, ARTIFACT_FACT_NAME);
    }

}