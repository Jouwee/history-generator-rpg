use std::{collections::HashMap, fmt::Display};

use crate::{commons::{history_vec::Id, rng::Rng}, engine::geometry::Coord2};

use super::{battle_simulator::BattleResult, world::{ArtifactId, World}};

// Speed of rumor spread, in units per year
const SPEED_OF_RUMORS: f32 = 5.;

#[derive(Debug, Clone, Copy)]
pub(crate) struct WorldEventDate {
    pub(crate) year: u32
}

impl Display for WorldEventDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.year)
    }
}

pub(crate) struct WorldEvents {
    base: Vec<WorldEvent>,
    by_unit: HashMap<Id, Vec<usize>>,
    empty_index: Vec<usize>
}

impl WorldEvents {

    pub(crate) fn new() -> WorldEvents {
        return WorldEvents {
            base: Vec::new(),
            by_unit: HashMap::new(),
            empty_index: Vec::new()
        }
    }

    pub(crate) fn push(&mut self, date: WorldEventDate, location: Coord2, event: WorldEventEnum) {
        let i = self.base.len();
        for unit in event.get_units().iter() {
            let entry = self.by_unit.entry(*unit).or_insert(Vec::new());
            entry.push(i)
        }
        self.base.push(WorldEvent { date, location, event });
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &WorldEvent> {
        self.base.iter()
    }

    pub(crate) fn iter_unit(&self, unit_id: &Id) -> impl Iterator<Item = &WorldEvent> {
        let idxs = self.by_unit.get(&unit_id);
        let vec;
        match idxs {
            Some(idxs) => vec = idxs,
            None => vec = &self.empty_index
        }
        vec.iter().map(|i| self.base.get(*i).unwrap())
    }

    pub(crate) fn get(&self, i: usize) -> Option<&WorldEvent> {
        self.base.get(i)
    }

    pub(crate) fn find_rumor(&self, rng: &Rng, world: &World, date: WorldEventDate, position: Coord2) -> Option<(usize, &WorldEvent)> {
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
pub(crate) struct WorldEvent {
    pub(crate) date: WorldEventDate,
    pub(crate) location: Coord2,
    pub(crate) event: WorldEventEnum,
}

impl WorldEvent {
    
    pub(crate) fn importance_factor(&self, world: &World) -> f32 {
        match &self.event {
            WorldEventEnum::ArtifactCreated(_evt) => 0.1,
            WorldEventEnum::ArtifactPossession(_evt) => 0.,
            WorldEventEnum::Battle(_evt) => 0.1,
            WorldEventEnum::PeaceDeclared(_evt) => 0.3,
            WorldEventEnum::WarDeclared(_evt) => 0.3,
            WorldEventEnum::SettlementFounded(_evt) => 0.01,
            WorldEventEnum::NewSettlementLeader(_evt) => 0.1,
            WorldEventEnum::Marriage(evt) => 0.01 * Self::creature_importance(world, evt.creature1_id).max(Self::creature_importance(world, evt.creature2_id)),
            WorldEventEnum::PersonBorn(_) => 0.0,
            WorldEventEnum::PersonDeath(evt) => {
                let cause_of_death_multiplier = match evt.cause_of_death {
                    CauseOfDeath::NaturalCauses => 0.2,
                    CauseOfDeath::KilledInBattle(killer, weapon) => {
                        let mut mult = 0.6;
                        if let Some(killer) = killer {
                            mult += 0.2 * Self::creature_importance(world, killer);
                        }
                        if weapon.is_some() {
                            mult += 0.2;
                        }
                        return mult
                    }
                };
                cause_of_death_multiplier * Self::creature_importance(world, evt.creature)
            },
        }
    }

    fn creature_importance(world: &World, id: Id) -> f32 {
        return 1.;
        // TODO:
        // let creature = world.creatures.get(&id).unwrap();
        // match creature.importance {
        //     super::creature::Importance::Important => 1.,
        //     super::creature::Importance::Unimportant => 0.5,
        //     super::creature::Importance::Unknown => 0.,
        // }
    }

}

#[derive(Debug, Clone)]
pub(crate) enum WorldEventEnum {
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
    pub(crate) fn get_units(&self) -> Vec<Id> {
        match self {
            Self::SettlementFounded(evt) => vec!(evt.unit_id),
            Self::NewSettlementLeader(evt) => vec!(evt.unit_id),
            Self::Battle(evt) => vec!(evt.battle_result.0.location_unit),
            _ => vec!()
        }
    }

    pub(crate) fn get_creatures(&self) -> Vec<Id> {
        let options = match self {
            Self::PersonBorn(evt) => vec!(Some(evt.creature_id)),
            Self::PersonDeath(evt) => {
                match evt.cause_of_death {
                    CauseOfDeath::NaturalCauses => vec!(Some(evt.creature)),
                    CauseOfDeath::KilledInBattle(killer, _) => vec!(Some(evt.creature), killer)
                }
            },
            Self::ArtifactPossession(evt) => vec!(Some(evt.creature)),
            Self::Marriage(evt) => vec!(Some(evt.creature1_id), Some(evt.creature2_id)),
            Self::NewSettlementLeader(evt) => vec!(Some(evt.new_leader_id)),
            _ => vec!()
        };
        options.iter().filter(|v| v.is_some()).map(|v| v.unwrap()).collect()
    }

    pub(crate) fn get_artifacts(&self) -> Vec<ArtifactId> {
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
pub(crate) struct SimplePersonEvent {
    pub(crate) creature_id: Id,
}

#[derive(Debug, Clone)]
pub(crate) struct CreatureDeathEvent {
    pub(crate) creature: Id,
    pub(crate) cause_of_death: CauseOfDeath 
}

#[derive(Debug, Clone)]
pub(crate) enum CauseOfDeath {
    NaturalCauses,
    KilledInBattle(/* killer */ Option<Id>, /* Item used */ Option<ArtifactId>)
}

#[derive(Debug, Clone)]
pub(crate) struct MarriageEvent {
    pub(crate) creature1_id: Id,
    pub(crate) creature2_id: Id,
}

#[derive(Debug, Clone)]
pub(crate) struct SettlementFoundedEvent {
    pub(crate) founder_id: Id,
    pub(crate) unit_id: Id,
}

#[derive(Debug, Clone)]
pub(crate) struct NewSettlementLeaderEvent {
    pub(crate) new_leader_id: Id,
    pub(crate) unit_id: Id,
}

#[derive(Debug, Clone)]
pub(crate) struct WarDeclaredEvent {
    pub(crate) faction1_id: Id,
    pub(crate) faction2_id: Id,
}

#[derive(Debug, Clone)]
pub(crate) struct PeaceDeclaredEvent {
    pub(crate) faction1_id: Id,
    pub(crate) faction2_id: Id,
}

#[derive(Debug, Clone)]
pub(crate) struct BattleEvent {
    pub(crate) battle_result: (BattleResult, BattleResult)
}

#[derive(Debug, Clone)]
pub(crate) struct ArtifactEvent {
    pub(crate) item: ArtifactId
}

#[derive(Debug, Clone)]
pub(crate) struct ArtifactPossesionEvent {
    pub(crate) item: ArtifactId,
    pub(crate) creature: Id
}
