use std::cell::Ref;

use crate::{commons::xp_table::xp_to_level, world::{creature::{Creature, CreatureId, Profession}, site::{Site, SiteId}, world::World}, Rng};

pub(crate) struct BattleSimulator {
}

const CIVILIAN_FIGHT_CHANCE: f32 = 0.1;

impl BattleSimulator {

    pub(crate) fn simulate_attack(attacker_id: SiteId, attacker: &Site, defender_id: SiteId, defender: &Site, rng: &mut Rng, world: &World) -> Battle {
        let mut battle = Battle {
            log: Vec::new(),
            deaths: Vec::new(),
            xp_add: Vec::new()
        };

        let mut creatures = Vec::new();

        for id in attacker.creatures.iter() {
            let creature = world.creatures.get(id);
            creatures.push(BattleCreature { id: *id, site_id: attacker_id, creature, hp: 100., team: 0, tactic: Tactic::Fight })
        }

        for id in defender.creatures.iter() {
            let creature = world.creatures.get(id);
            let tactic = match creature.profession {
                Profession::Guard | Profession::Beast | Profession::Bandit | Profession::Ruler => Tactic::Fight,
                Profession::None => Tactic::Hide,
                _ => { 
                    if rng.rand_chance(CIVILIAN_FIGHT_CHANCE) {
                        Tactic::Fight
                    } else {
                        Tactic::Hide
                    }
                },
            };
            creatures.push(BattleCreature { id: *id, site_id: defender_id, creature, hp: 100., team: 1, tactic })
        }

        let max_turns = rng.randu_range(5, 15) * creatures.len();
        for i in 0..max_turns {
            let turn_index = i % creatures.len();
            if creatures.len() == 0 {
                battle.log.push(String::from("No one's left to fight"));
            }
            let creature = creatures.get(turn_index).expect("Is from range");

            if let Tactic::Hide = creature.tactic {
                continue;
            }

            let adversary = Self::choose_adversary(&creatures, rng, turn_index, creature.team);

            let adversary = match adversary {
                None => {
                    battle.log.push(String::from("No one's left from the other team"));
                    break
                },
                Some(adversary) => adversary
            };

            let damage = rng.randf_range(5., 15.) * xp_to_level(creature.creature.experience) as f32;
            
            let adversary_creature = creatures.get_mut(adversary).expect("Is from range");
            adversary_creature.hp -= damage;
            let hp = adversary_creature.hp;

            battle.log.push(format!("{:?} attacked {:?}", turn_index, adversary));

            if hp <= 0. {
                battle.log.push(format!("{:?} was killed by {:?}", adversary, turn_index));

                let creature = creatures.get(turn_index).expect("Is from range");

                let adversary_creature = creatures.get(adversary).expect("Is from range");
                battle.deaths.push((adversary_creature.id, adversary_creature.site_id, creature.id));
                battle.xp_add.push((creature.id, 50 * xp_to_level(adversary_creature.creature.experience) as u32));

                creatures.remove(adversary);
            }
        }

        return battle
    }

    fn choose_adversary(creatures: &Vec<BattleCreature>, rng: &mut Rng, turn_index: usize, team: u8) -> Option<usize> {
        let mut candidates = Vec::new();
        for i in 0..creatures.len() {
            if i == turn_index {
                continue;
            }
            let creature = creatures.get(i).expect("In range");
            if creature.team == team {
                continue;
            }
            if let Tactic::Hide = creature.tactic {
                if rng.rand_chance(0.9) {
                    continue;
                }
            }
            candidates.push(i);
        }
        return rng.item(&candidates).copied();
    }

}

struct BattleCreature<'a> {
    id: CreatureId,
    site_id: SiteId,
    creature: Ref<'a, Creature>,
    tactic: Tactic,
    hp: f32,
    team: u8,
}

enum Tactic {
    Fight,
    Hide
}

pub(crate) struct Battle {
    pub(crate) log: Vec<String>,
    pub(crate) deaths: Vec<(CreatureId, SiteId, CreatureId)>,
    pub(crate) xp_add: Vec<(CreatureId, u32)>,
}
