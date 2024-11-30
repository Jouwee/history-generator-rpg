use std::{collections::HashMap, fmt::Display};

use crate::{commons::history_vec::Id, BattleResult};

#[derive(Clone, Copy)]
pub struct WorldEventDate {
    pub year: u32
}

impl Display for WorldEventDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.year)
    }
}

pub struct WorldEvents {
    base: Vec<WorldEvent>,
    by_settlement: HashMap<Id, Vec<usize>>,
    empty_index: Vec<usize>
}

impl WorldEvents {
    pub fn new() -> WorldEvents {
        return WorldEvents {
            base: Vec::new(),
            by_settlement: HashMap::new(),
            empty_index: Vec::new()
        }
    }

    pub fn push(&mut self, date: WorldEventDate, event: WorldEventEnum) {
        let i = self.base.len();
        for settlement in event.get_settlements().iter() {
            let entry = self.by_settlement.entry(*settlement).or_insert(Vec::new());
            entry.push(i)
        }
        self.base.push(WorldEvent { date, event });
    }

    pub fn iter(&self) -> impl Iterator<Item = &WorldEvent> {
        self.base.iter()
    }

    pub fn iter_settlement(&self, settlement_id: &Id) -> impl Iterator<Item = &WorldEvent> {
        let idxs = self.by_settlement.get(&settlement_id);
        let vec;
        match idxs {
            Some(idxs) => vec = idxs,
            None => vec = &self.empty_index
        }
        vec.iter().map(|i| self.base.get(*i).unwrap())
    }

}

pub struct WorldEvent {
    pub date: WorldEventDate,
    pub event: WorldEventEnum,
}

pub enum WorldEventEnum {
    PersonBorn(SimplePersonEvent),
    PersonDeath(SimplePersonEvent),
    Marriage(MarriageEvent),
    NewSettlementLeader(NewSettlementLeaderEvent),
    SettlementFounded(SettlementFoundedEvent),
    WarDeclared(WarDeclaredEvent),
    PeaceDeclared(PeaceDeclaredEvent),
    Siege(SiegeEvent),
}

impl WorldEventEnum {
    fn get_settlements(&self) -> Vec<Id> {
        match self {
            Self::SettlementFounded(evt) => vec!(evt.settlement_id),
            Self::NewSettlementLeader(evt) => vec!(evt.settlement_id),
            Self::Siege(evt) => vec!(evt.settlement1_id, evt.settlement2_id),
            _ => vec!()
        }
    }
}
pub struct SimplePersonEvent {
    pub person_id: Id,
}

pub struct MarriageEvent {
    pub person1_id: Id,
    pub person2_id: Id,
}

pub struct SettlementFoundedEvent {
    pub founder_id: Id,
    pub settlement_id: Id,
}

pub struct NewSettlementLeaderEvent {
    pub new_leader_id: Id,
    pub settlement_id: Id,
}

pub struct WarDeclaredEvent {
    pub faction1_id: Id,
    pub faction2_id: Id,
}

pub struct PeaceDeclaredEvent {
    pub faction1_id: Id,
    pub faction2_id: Id,
}

pub struct SiegeEvent {
    pub faction1_id: Id,
    pub faction2_id: Id,
    pub settlement1_id: Id,
    pub settlement2_id: Id,
    pub battle_result: BattleResult,
}