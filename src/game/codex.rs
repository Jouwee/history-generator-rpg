use std::{collections::HashMap, slice::Iter};

use crate::{commons::bitmask::{bitmask_get, bitmask_set}, world::{creature::CreatureId, item::ItemId, unit::UnitId}};

pub(crate) struct Codex {
    creatures: HashMap<CreatureId, CreatureCodex>,
    artifacts: HashMap<ItemId, ArtifactCodex>,
    units: HashMap<UnitId, UnitCodex>,
    quests: Vec<Quest>,
}

impl Codex {

    pub(crate) fn new() -> Self {
        Codex {
            creatures: HashMap::new(),
            artifacts: HashMap::new(),
            units: HashMap::new(),
            quests: Vec::new()
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
            self.artifacts.insert(*artifact_id, ArtifactCodex { events: Vec::new() });    
        }
        return self.artifacts.get_mut(artifact_id).expect("Just inserted");
    }

    pub(crate) fn units(&self) -> std::collections::hash_map::Keys<'_, UnitId, UnitCodex> {
        return self.units.keys();
    }

    pub(crate) fn unit(&self, unit_id: &UnitId) -> Option<&UnitCodex> {
        return self.units.get(unit_id);
    }

    pub(crate) fn unit_mut(&mut self, unit_id: &UnitId) -> &mut UnitCodex {
        if !self.units.contains_key(unit_id) {
            self.units.insert(*unit_id, UnitCodex { });    
        }
        return self.units.get_mut(unit_id).expect("Just inserted");
    }

    pub(crate) fn add_quest(&mut self, quest: Quest) {
        // Makes sure the basic info about the quest is known
        match &quest.objective {
            QuestObjective::KillVarningr(creature_id) => {
                let creature = self.creature_mut(creature_id);
                creature.add_name();
            },
            _ => {}
        }

        self.quests.push(quest);
    }

    pub(crate) fn quests(&self) -> impl Iterator<Item = &Quest> {
        self.quests.iter()
    }

    pub(crate) fn quests_mut(&mut self) -> impl Iterator<Item = &mut Quest> {
        self.quests.iter_mut()
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

    pub(crate) fn add_event(&mut self, event: usize) {
        self.events.push(event)
    }

    pub(crate) fn events(&'_ self) -> Iter<'_, usize> {
        return self.events.iter()
    }

}

pub(crate) struct ArtifactCodex {
    events: Vec<usize>,
}

impl ArtifactCodex {

    pub(crate) fn add_event(&mut self, event: usize) {
        self.events.push(event)
    }

    pub(crate) fn events(&'_ self) -> Iter<'_, usize> {
        return self.events.iter()
    }

}


#[derive(Clone, Debug)]
pub(crate) struct Quest {
    pub(crate) status: QuestStatus,
    pub(crate) quest_giver: CreatureId,
    pub(crate) objective: QuestObjective
}

impl Quest {

    pub(crate) fn new(quest_giver: CreatureId, objective: QuestObjective) -> Self {
        return Self {
            quest_giver,
            objective,
            status: QuestStatus::InProgress
        }
    }

}

#[derive(Clone, Debug)]
pub(crate) enum QuestObjective {
    /// Kill a varningr
    KillVarningr(CreatureId),
    /// Kill wolves
    KillWolves(UnitId),
    /// Kill bandits
    KillBandits(UnitId),
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum QuestStatus {
    InProgress,
    RewardPending,
    Complete
}

pub(crate) struct UnitCodex {
}
