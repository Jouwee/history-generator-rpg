use crate::{commons::{history_vec::Id, rng::Rng}, engine::geometry::Coord2, resources::resources::Resources};

use super::{history_sim::structs::World, settlement::Settlement, species::Species};

pub struct BattleForce {
    belligerent_faction: Option<Id>,
    belligerent_settlement: Option<Id>,
    units: Vec<Unit>,
    /* Ranges from 0.0 to 1.0. Morale of 0 means they'll flee, morale of 1. means fighting to the death. */
    morale: f32,
    total_combat_force: f32
}

#[derive(Clone)]
struct Unit {
    creature_id: Option<Id>,
    combat_power: f32,
    health: f32,
}

impl Unit {

    fn new(combat_power: f32, health: f32) -> Unit {
        Unit { creature_id: None, combat_power, health }
    }

    fn from_creature(id: &Id, species: &Species) -> Unit {
        Unit { creature_id: Some(id.clone()), combat_power: species.attributes.simplified_offensive_power(), health: species.attributes.simplified_health() }
    }

}

impl BattleForce {

    pub fn from_attacking_settlement(world: &World, resources: &Resources, settlement_id: Id, settlement: &Settlement) -> BattleForce {
        return Self::from_defending_settlement(world, resources, settlement_id, settlement)
    }

    pub fn from_defending_settlement(world: &World, resources: &Resources, settlement_id: Id, settlement: &Settlement) -> BattleForce {
        let mut units: Vec<Unit> = (0..settlement.military.trained_soldiers).map(|_| Unit::new(10., 20.)).collect();
        // TODO:
        // for (id, creature) in world.creatures.iter() {
        //     let creature = creature.try_borrow();
        //     if let Ok(creature) = creature {
        //         // TODO: Improve performance
        //         if creature.position == settlement.xy.to_coord() {
        //             let species = resources.species.get(&creature.species);
        //             units.push(Unit::from_creature(id, species));
        //         }
        //     }
        // }
        BattleForce::new(Some(settlement.faction_id), Some(settlement_id), units)
    }

    // pub fn from_creatures(resources: &Resources, creatures: Vec<&Person>) -> BattleForce {
    //     BattleForce::new(None, None, creatures.iter().map(|creature| Unit::from_creature(&creature.id, resources.species.get(&creature.species))).collect())
    // }

    fn new(belligerent_faction: Option<Id>, belligerent_settlement: Option<Id>, units: Vec<Unit>) -> BattleForce {
        let total_combat_force = units.iter().fold(0., |acc, unit| acc + unit.combat_power);
        BattleForce {
            belligerent_faction,
            belligerent_settlement,
            units,
            morale: 0.6,
            total_combat_force
        }
    }

    pub fn battle(&mut self, another: &mut BattleForce, rng: &mut Rng, location: Coord2, location_settlement: Id) -> (BattleResult, BattleResult) {
        let mut result_self = BattleResult::new(location, location_settlement);
        result_self.belligerent_faction = self.belligerent_faction;
        result_self.belligerent_settlement = self.belligerent_settlement;
        result_self.creature_participants = self.units.iter().filter_map(|unit| unit.creature_id).collect();
        let mut result_another = BattleResult::new(location, location_settlement);
        result_another.belligerent_faction = another.belligerent_faction;
        result_another.belligerent_settlement = another.belligerent_settlement;
        result_another.creature_participants = another.units.iter().filter_map(|unit| unit.creature_id).collect();
        loop {
            if self.units.len() == 0 {
                result_self.result = FinalResult::Defeat;
                break
            }
            if another.units.len() == 0 {
                result_another.result = FinalResult::Defeat;
                break
            }
            if self.morale <= 0.0 {
                result_self.result = FinalResult::Flee;
                break
            }
            if another.morale <= 0.0 {
                result_another.result = FinalResult::Flee;
                break
            }
            for unit in self.units.iter() {
                let target_i = rng.randu_range(0, another.units.len());
                let target = another.units.get(target_i).unwrap().clone();
                // Hit
                if rng.randf_range(0., unit.combat_power) > rng.randf_range(0., target.combat_power) {
                    let killed;
                    let damage = rng.randf_range(0., unit.combat_power);
                    {
                        let target = another.units.get_mut(target_i).unwrap();
                        target.health -= damage;
                        killed = target.health <= 0.;
                    }
                    if killed {
                        another.units.remove(target_i);
                        another.morale -= target.combat_power / another.total_combat_force;
                        if let Some(id) = target.creature_id {
                            result_another.creature_casualties.push((id.clone(), unit.creature_id));
                        } else {
                            result_another.army_casualties += 1;
                        }
                        if another.units.len() == 0 {
                            break
                        }
                    }
                }
            }
            for unit in another.units.iter() {
                let target_i = rng.randu_range(0, self.units.len());
                let target = self.units.get(target_i).unwrap().clone();
                if rng.randf_range(0., unit.combat_power) > rng.randf_range(0., target.combat_power) {
                    let killed;
                    let damage = rng.randf_range(0., unit.combat_power);
                    {
                        let target = self.units.get_mut(target_i).unwrap();
                        target.health -= damage;
                        killed = target.health <= 0.;
                    }
                    if killed {
                        self.units.remove(target_i);
                        self.morale -= target.combat_power / self.total_combat_force;
                        if let Some(id) = target.creature_id {
                            result_self.creature_casualties.push((id.clone(), unit.creature_id));
                        } else {
                            result_self.army_casualties += 1;
                        }
                        if self.units.len() == 0 {
                            break
                        }
                    }
                }
            }

            // Every turn the battle drags, there's a chance of killing civilians
            if let Some(settlement) = another.belligerent_settlement {
                if location_settlement == settlement && rng.rand_chance(0.5) {
                    result_another.civilian_casualties += 1;
                }
            }

            if let Some(settlement) = self.belligerent_settlement {
                if location_settlement == settlement && rng.rand_chance(0.5) {
                    result_self.civilian_casualties += 1;
                }
            }

        }
        return (result_self, result_another)
    }

}



#[derive(Debug, Clone)]
pub struct BattleResult {
    pub belligerent_faction: Option<Id>,
    pub belligerent_settlement: Option<Id>,
    pub location: Coord2,
    pub location_settlement: Id,
    pub result: FinalResult,
    pub creature_participants: Vec<Id>,
    // ID of creature killed, ID of killer (might be a generic creature)
    pub creature_casualties: Vec<(Id, Option<Id>)>,
    pub army_casualties: u32,
    pub civilian_casualties: u32,
}

impl BattleResult {
    fn new(location: Coord2, location_settlement: Id) -> BattleResult {
        BattleResult {
            belligerent_faction: None,
            belligerent_settlement: None,
            location,
            location_settlement,
            result: FinalResult::Victory,
            creature_participants: Vec::new(),
            creature_casualties: Vec::new(),
            army_casualties: 0,
            civilian_casualties: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FinalResult {
    Flee,
    Defeat,
    Victory
}