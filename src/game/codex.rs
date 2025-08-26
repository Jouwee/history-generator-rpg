use std::{collections::HashMap, slice::Iter};

use serde::{Deserialize, Serialize};

use crate::{commons::bitmask::{bitmask_get, bitmask_set}, world::{creature::CreatureId, item::ItemId, site::SiteId}};

#[derive(Serialize, Deserialize)]
pub(crate) struct Codex {
    creatures: HashMap<CreatureId, CreatureCodex>,
    artifacts: HashMap<ItemId, ArtifactCodex>,
    sites: HashMap<SiteId, SiteCodex>,
    quests: Vec<Quest>,
}

impl Codex {

    pub(crate) fn new() -> Self {
        Codex {
            creatures: HashMap::new(),
            artifacts: HashMap::new(),
            sites: HashMap::new(),
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

    pub(crate) fn sites(&self) -> std::collections::hash_map::Keys<'_, SiteId, SiteCodex> {
        return self.sites.keys();
    }

    pub(crate) fn site(&self, site_id: &SiteId) -> Option<&SiteCodex> {
        return self.sites.get(site_id);
    }

    pub(crate) fn site_mut(&mut self, site_id: &SiteId) -> &mut SiteCodex {
        if !self.sites.contains_key(site_id) {
            self.sites.insert(*site_id, SiteCodex { });    
        }
        return self.sites.get_mut(site_id).expect("Just inserted");
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

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
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


#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum QuestObjective {
    /// Kill a varningr
    KillVarningr(CreatureId),
    /// Kill wolves
    KillWolves(SiteId),
    /// Kill bandits
    KillBandits(SiteId),
}


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum QuestStatus {
    InProgress,
    RewardPending,
    Complete
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SiteCodex {
}
