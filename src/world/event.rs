use std::{collections::HashMap, fmt::Display};

use crate::{commons::{history_vec::Id, rng::Rng}, engine::geometry::Coord2};

use super::{battle_simulator::BattleResult, world::{ArtifactId, World}};

// Speed of rumor spread, in units per year
const SPEED_OF_RUMORS: f32 = 5.;

#[derive(Debug, Clone, Copy)]
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

    pub fn push(&mut self, date: WorldEventDate, location: Coord2, event: WorldEventEnum) {
        let i = self.base.len();
        for settlement in event.get_settlements().iter() {
            let entry = self.by_settlement.entry(*settlement).or_insert(Vec::new());
            entry.push(i)
        }
        self.base.push(WorldEvent { date, location, event });
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

    pub fn get(&self, i: usize) -> Option<&WorldEvent> {
        self.base.get(i)
    }

    pub fn find_rumor(&self, rng: &Rng, world: &World, date: WorldEventDate, position: Coord2) -> Option<(usize, &WorldEvent)> {
        let mut rng = rng.derive("rumor");
        for (i, event) in self.base.iter().enumerate().rev() {
            let dist = event.location.dist_squared(&position).sqrt();
            let evt_date = event.date;
            let age = (date.year - evt_date.year) as f32;
            if (dist / SPEED_OF_RUMORS) > age {
                continue
            }
            let dist_factor = (1. - dist / 128.).max(0.001);
            let age_factor = (1. - age / 50.).max(0.001);
            let importance = event.importance_factor(world);
            let chance_to_have_heard = importance * dist_factor * age_factor;
            if rng.rand_chance(chance_to_have_heard) {
                return Some((i, event))
            }
        }
        return None
    }

}

#[derive(Debug, Clone)]
pub struct WorldEvent {
    pub date: WorldEventDate,
    pub location: Coord2,
    pub event: WorldEventEnum,
}

impl WorldEvent {
    
    pub fn importance_factor(&self, world: &World) -> f32 {
        match &self.event {
            WorldEventEnum::ArtifactCreated(_evt) => 0.1,
            WorldEventEnum::ArtifactPossession(_evt) => 0.,
            WorldEventEnum::Battle(_evt) => 0.1,
            WorldEventEnum::PeaceDeclared(_evt) => 0.3,
            WorldEventEnum::WarDeclared(_evt) => 0.3,
            WorldEventEnum::SettlementFounded(_evt) => 0.01,
            WorldEventEnum::NewSettlementLeader(_evt) => 0.1,
            WorldEventEnum::Marriage(evt) => 0.01 * Self::person_importance(world, evt.person1_id).max(Self::person_importance(world, evt.person2_id)),
            WorldEventEnum::PersonBorn(_) => 0.0,
            WorldEventEnum::PersonDeath(evt) => {
                let cause_of_death_multiplier = match evt.cause_of_death {
                    CauseOfDeath::NaturalCauses => 0.2,
                    CauseOfDeath::KilledInBattle(killer, weapon) => {
                        let mut mult = 0.6;
                        if let Some(killer) = killer {
                            mult += 0.2 * Self::person_importance(world, killer);
                        }
                        if weapon.is_some() {
                            mult += 0.2;
                        }
                        return mult
                    }
                };
                cause_of_death_multiplier * Self::person_importance(world, evt.creature)
            },
        }
    }

    fn person_importance(world: &World, id: Id) -> f32 {
        let person = world.people.get(&id).unwrap();
        match person.importance {
            super::person::Importance::Important => 1.,
            super::person::Importance::Unimportant => 0.5,
            super::person::Importance::Unknown => 0.,
        }
    }

}

#[derive(Debug, Clone)]
pub enum WorldEventEnum {
    PersonBorn(SimplePersonEvent),
    PersonDeath(CreatureDeathEvent),
    Marriage(MarriageEvent),
    NewSettlementLeader(NewSettlementLeaderEvent),
    SettlementFounded(SettlementFoundedEvent),
    WarDeclared(WarDeclaredEvent),
    PeaceDeclared(PeaceDeclaredEvent),
    Battle(BattleEvent),
    ArtifactCreated(ArtifactEvent),
    ArtifactPossession(ArtifactPossesionEvent),
}

impl WorldEventEnum {
    pub fn get_settlements(&self) -> Vec<Id> {
        match self {
            Self::SettlementFounded(evt) => vec!(evt.settlement_id),
            Self::NewSettlementLeader(evt) => vec!(evt.settlement_id),
            Self::Battle(evt) => vec!(evt.battle_result.0.location_settlement),
            _ => vec!()
        }
    }

    pub fn get_creatures(&self) -> Vec<Id> {
        let options = match self {
            Self::PersonBorn(evt) => vec!(Some(evt.person_id)),
            Self::PersonDeath(evt) => {
                match evt.cause_of_death {
                    CauseOfDeath::NaturalCauses => vec!(Some(evt.creature)),
                    CauseOfDeath::KilledInBattle(killer, _) => vec!(Some(evt.creature), killer)
                }
            },
            Self::ArtifactPossession(evt) => vec!(Some(evt.person)),
            Self::Marriage(evt) => vec!(Some(evt.person1_id), Some(evt.person2_id)),
            Self::NewSettlementLeader(evt) => vec!(Some(evt.new_leader_id)),
            _ => vec!()
        };
        options.iter().filter(|v| v.is_some()).map(|v| v.unwrap()).collect()
    }

    pub fn get_artifacts(&self) -> Vec<ArtifactId> {
        let options = match self {
            Self::PersonDeath(evt) => {
                match evt.cause_of_death {
                    CauseOfDeath::NaturalCauses => vec!(),
                    CauseOfDeath::KilledInBattle(_, weapon) => vec!(weapon)
                }
            },
            Self::ArtifactCreated(evt) => vec!(Some(evt.item)),
            Self::ArtifactPossession(evt) => vec!(Some(evt.item)),
            _ => vec!()
        };
        options.iter().filter(|v| v.is_some()).map(|v| v.unwrap()).collect()
    }
}

#[derive(Debug, Clone)]
pub struct SimplePersonEvent {
    pub person_id: Id,
}

#[derive(Debug, Clone)]
pub struct CreatureDeathEvent {
    pub creature: Id,
    pub cause_of_death: CauseOfDeath 
}

#[derive(Debug, Clone)]
pub enum CauseOfDeath {
    NaturalCauses,
    KilledInBattle(/* killer */ Option<Id>, /* Item used */ Option<ArtifactId>)
}

#[derive(Debug, Clone)]
pub struct MarriageEvent {
    pub person1_id: Id,
    pub person2_id: Id,
}

#[derive(Debug, Clone)]
pub struct SettlementFoundedEvent {
    pub founder_id: Id,
    pub settlement_id: Id,
}

#[derive(Debug, Clone)]
pub struct NewSettlementLeaderEvent {
    pub new_leader_id: Id,
    pub settlement_id: Id,
}

#[derive(Debug, Clone)]
pub struct WarDeclaredEvent {
    pub faction1_id: Id,
    pub faction2_id: Id,
}

#[derive(Debug, Clone)]
pub struct PeaceDeclaredEvent {
    pub faction1_id: Id,
    pub faction2_id: Id,
}

#[derive(Debug, Clone)]
pub struct BattleEvent {
    pub battle_result: (BattleResult, BattleResult)
}

#[derive(Debug, Clone)]
pub struct ArtifactEvent {
    pub item: ArtifactId
}

#[derive(Debug, Clone)]
pub struct ArtifactPossesionEvent {
    pub item: ArtifactId,
    pub person: Id
}
