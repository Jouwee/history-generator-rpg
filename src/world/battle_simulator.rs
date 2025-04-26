use crate::{commons::{history_vec::Id, rng::Rng}, engine::geometry::Coord2, resources::resources::Resources};

use super::{history_sim::structs::World, species::Species};

pub(crate) struct BattleForce {
    belligerent_faction: Option<Id>,
    belligerent_unit: Option<Id>,
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

    // pub(crate) fn from_attacking_unit(world: &World, resources: &Resources, unit_id: Id, unit: &Settlement) -> BattleForce {
    //     return Self::from_defending_unit(world, resources, unit_id, unit)
    // }

    // pub(crate) fn from_defending_unit(world: &World, resources: &Resources, unit_id: Id, unit: &Settlement) -> BattleForce {
    //     let mut units: Vec<Unit> = (0..unit.military.trained_soldiers).map(|_| Unit::new(10., 20.)).collect();
    //     // TODO:
    //     // for (id, creature) in world.creatures.iter() {
    //     //     let creature = creature.try_borrow();
    //     //     if let Ok(creature) = creature {
    //     //         // TODO: Improve performance
    //     //         if creature.position == unit.xy.to_coord() {
    //     //             let species = resources.species.get(&creature.species);
    //     //             units.push(Unit::from_creature(id, species));
    //     //         }
    //     //     }
    //     // }
    //     BattleForce::new(Some(unit.faction_id), Some(unit_id), units)
    // }

    // pub(crate) fn from_creatures(resources: &Resources, creatures: Vec<&Person>) -> BattleForce {
    //     BattleForce::new(None, None, creatures.iter().map(|creature| Unit::from_creature(&creature.id, resources.species.get(&creature.species))).collect())
    // }

    fn new(belligerent_faction: Option<Id>, belligerent_unit: Option<Id>, units: Vec<Unit>) -> BattleForce {
        let total_combat_force = units.iter().fold(0., |acc, unit| acc + unit.combat_power);
        BattleForce {
            belligerent_faction,
            belligerent_unit,
            units,
            morale: 0.6,
            total_combat_force
        }
    }

    pub(crate) fn battle(&mut self, another: &mut BattleForce, rng: &mut Rng, location: Coord2, location_unit: Id) -> (BattleResult, BattleResult) {
        let mut result_self = BattleResult::new(location, location_unit);
        result_self.belligerent_faction = self.belligerent_faction;
        result_self.belligerent_unit = self.belligerent_unit;
        result_self.creature_participants = self.units.iter().filter_map(|unit| unit.creature_id).collect();
        let mut result_another = BattleResult::new(location, location_unit);
        result_another.belligerent_faction = another.belligerent_faction;
        result_another.belligerent_unit = another.belligerent_unit;
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
            if let Some(unit) = another.belligerent_unit {
                if location_unit == unit && rng.rand_chance(0.5) {
                    result_another.civilian_casualties += 1;
                }
            }

            if let Some(unit) = self.belligerent_unit {
                if location_unit == unit && rng.rand_chance(0.5) {
                    result_self.civilian_casualties += 1;
                }
            }

        }
        return (result_self, result_another)
    }

}



#[derive(Debug, Clone)]
pub(crate) struct BattleResult {
    pub(crate) belligerent_faction: Option<Id>,
    pub(crate) belligerent_unit: Option<Id>,
    pub(crate) location: Coord2,
    pub(crate) location_unit: Id,
    pub(crate) result: FinalResult,
    pub(crate) creature_participants: Vec<Id>,
    // ID of creature killed, ID of killer (might be a generic creature)
    pub(crate) creature_casualties: Vec<(Id, Option<Id>)>,
    pub(crate) army_casualties: u32,
    pub(crate) civilian_casualties: u32,
}

impl BattleResult {
    fn new(location: Coord2, location_unit: Id) -> BattleResult {
        BattleResult {
            belligerent_faction: None,
            belligerent_unit: None,
            location,
            location_unit,
            result: FinalResult::Victory,
            creature_participants: Vec::new(),
            creature_casualties: Vec::new(),
            army_casualties: 0,
            civilian_casualties: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum FinalResult {
    Flee,
    Defeat,
    Victory
}