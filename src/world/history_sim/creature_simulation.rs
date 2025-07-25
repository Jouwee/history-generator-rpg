use std::cell::Ref;

use crate::{commons::rng::Rng, history_trace, resources::resources::Resources, warn, world::{creature::{CauseOfDeath, Creature, CreatureGender, CreatureId, Goal, Profession}, date::WorldDate, event::Event, history_sim::battle_simulator::BattleSimulator, item::{Item, ItemId}, plot::{Plot, PlotGoal}, unit::{Unit, UnitId}, world::World}};

pub(crate) struct CreatureSimulation {}

#[derive(Debug)]
pub(crate) enum CreatureSideEffect {
    None,
    Death(CauseOfDeath),
    HaveChild,
    LookForMarriage,
    LookForNewJob,
    MakeArtifact,
    ComissionArtifact,
    ArtisanLookingForComission,
    BecomeBandit,
    AttackNearbyUnits,
    StartPlot(PlotGoal),
    FindSupportersForPlot,
    ExecutePlot,
}

const YEARLY_CHANCE_MARRY: f32 = 0.4;
const YEARLY_CHANCE_CHILD_MULT: f32 = 1.0;
const CHANCE_TO_STARVE: f32 = 0.2;
const BASE_DISEASE_CHANCE: f32 = 0.0015;
const CHANCE_NEW_JOB: f32 = 0.005;
const CHANCE_MAKE_INSPIRED_ARTIFACT: f32 = 0.005;
const CHANCE_TO_COMISSION_ARTIFACT_ON_BDAY: f32 = 0.5;
const CHANCE_TO_BECOME_BANDIT: f32 = 0.005;

impl CreatureSimulation {

    // TODO: Smaller steps
    pub(crate) fn simulate_step_creature(_step: &WorldDate, now: &WorldDate, rng: &mut Rng, unit: &Unit, creature: &Creature, supported_plot: Option<Ref<Plot>>) -> CreatureSideEffect {
        let age = (*now - creature.birth).year();
        // Death by starvation
        if unit.resources.food <= 0. && rng.rand_chance(CHANCE_TO_STARVE) {
            return CreatureSideEffect::Death(CauseOfDeath::Starvation);
        }
        // Death by disease
        if rng.rand_chance(Self::chance_of_disease(now, &creature)) {
            return CreatureSideEffect::Death(CauseOfDeath::Disease);
        }

        if creature.sim_flag_is_great_beast() {
            if rng.rand_chance(YEARLY_CHANCE_BEAST_HUNT) {
                return CreatureSideEffect::AttackNearbyUnits;
            }
        }

        // Get a profession
        if creature.sim_flag_is_inteligent() {

            match supported_plot {
                None => {
                    for goal in creature.goals.iter() {
                        let plot = goal.as_plot_goal();
                        if let Some(plot) = plot {
                            // TODO(IhlgIYVA): Magic number
                            if rng.rand_chance(0.3) {
                                return CreatureSideEffect::StartPlot(plot);
                            }
                        }
                    }
                },
                Some(plot) => {
                    if rng.rand_chance(1. - plot.success_chance()) {
                        return CreatureSideEffect::FindSupportersForPlot;
                    } else {
                        return CreatureSideEffect::ExecutePlot;
                    }
                    
                }
            }

            if age >= 14 && creature.profession == Profession::None {
                return CreatureSideEffect::LookForNewJob;
            }
            if age >= 18 {
                // Have child
                if creature.gender.is_female() && creature.spouse.is_some()  {
                    if rng.rand_chance(Self::chance_of_child(now, creature, unit.resources.food, unit.creatures.len())) {
                        return CreatureSideEffect::HaveChild;
                    }
                }
                // Find a spouse
                if creature.spouse.is_none() {
                    if rng.rand_chance(YEARLY_CHANCE_MARRY) {
                        return CreatureSideEffect::LookForMarriage;
                    }
                }

                // Look for new job
                if !creature.profession.is_for_life() {
                    if rng.rand_chance(CHANCE_NEW_JOB) {
                        return CreatureSideEffect::LookForNewJob;
                    }
                    if rng.rand_chance(CHANCE_TO_BECOME_BANDIT) {
                        return CreatureSideEffect::BecomeBandit;
                    }
                }
                
                if creature.profession == Profession::Ruler && age % 10 == 0 {
                    if rng.rand_chance(CHANCE_TO_COMISSION_ARTIFACT_ON_BDAY) {
                        return CreatureSideEffect::ComissionArtifact;
                    }
                }

            }
        }

        if age >= 60 {
            // Death of old age
            if rng.rand_chance(Self::chance_of_death_by_old_age(age as f32)) {
                return CreatureSideEffect::Death(CauseOfDeath::OldAge);
            }
        }

        match creature.profession {
            Profession::Blacksmith => {
                if rng.rand_chance(CHANCE_MAKE_INSPIRED_ARTIFACT) {
                    return CreatureSideEffect::MakeArtifact;
                }
                return CreatureSideEffect::ArtisanLookingForComission;
            },
            Profession::Sculptor => {
                return CreatureSideEffect::ArtisanLookingForComission;
            },
            _ => ()
        }
        return CreatureSideEffect::None
    }

    fn chance_of_child(now: &WorldDate, creature: &Creature, unit_food_stock: f32, unit_population: usize) -> f32 {
        let food_excess_pct = unit_food_stock / unit_population as f32;
        let food_mult = (food_excess_pct - 1.).clamp(0.02, 1.);
        
        let children_mult = 1. - (creature.offspring.len() as f32 / 10.);
        let age = (*now - creature.birth).year() as f32;
        
        let fertility_mult = (0.96 as f32).powf(age - 18.) * (0.92 as f32).powf(age - 18.);

        return YEARLY_CHANCE_CHILD_MULT * fertility_mult * food_mult * children_mult;
    }

    fn chance_of_disease(now: &WorldDate, creature: &Creature) -> f32 {
        let age = (*now - creature.birth).year() as f32;
        // Children are more suceptible to disease
        if age < 18. {
            let boost = (age / 18.).powf(2.) + 1.;
            return BASE_DISEASE_CHANCE + (boost * BASE_DISEASE_CHANCE);
        }
        // Same as older people
        if age >= 40. {
            let boost = ((age - 40.) / 40.).powf(2.);
            return BASE_DISEASE_CHANCE + (boost * BASE_DISEASE_CHANCE);
        }
        return BASE_DISEASE_CHANCE;
    }

    fn chance_of_death_by_old_age(age: f32) -> f32 {
        return ((age - 60.) / 60.).powf(4.0).clamp(0., 1.)
    }

    pub(crate) fn have_child_with_spouse(now: &WorldDate, world: &World, rng: &mut Rng, creature_id: &CreatureId, creature: &mut Creature) -> Option<Creature> {
        let father_id = creature.spouse;
        if let Some(father_id) = father_id {
            let father = world.creatures.get(&father_id);
            let lineage = father.lineage.clone();
            let mut gender = CreatureGender::Male;
            if rng.rand_chance(0.5) {
                gender = CreatureGender::Female;
            }
            let child = Creature {
                birth: now.clone(),
                death: None,
                profession: Profession::None,
                lineage,
                mother: *creature_id,
                father: father_id,
                gender,
                offspring: Vec::new(),
                species: creature.species,
                spouse: None,
                details: None,
                experience: 0,
                sim_flags: father.sim_flags,
                relationships: vec!(),
                goals: vec!(),
                supports_plot: None,
            };
            return Some(child)
        }
        return None
    }

}

// Legendary beasts

const YEARLY_CHANCE_BEAST_HUNT: f32 = 0.8;
const HUNT_RADIUS_SQRD: f32 = 20.*20.;

pub(crate) fn attack_nearby_unit(world: &mut World, rng: &mut Rng, unit_id: UnitId, resources: &mut Resources) {
    let mut candidates = Vec::new();
    {
        let source_unit = world.units.get(&unit_id);
        for (id, unit) in world.units.iter_id_val::<UnitId>() {
            if id != unit_id {
                let unit = unit.borrow();
                if unit.creatures.len() > 0 && unit.xy.dist_squared(&source_unit.xy) < HUNT_RADIUS_SQRD {
                    candidates.push(id);
                    break;
                }
            }
        }
    }

    if let Some(target) = rng.item(&candidates) {
        let battle;
        {
            let unit = world.units.get(&unit_id);
            let target_unit = world.units.get(target);
            battle = BattleSimulator::simulate_attack(unit_id, &unit, *target, &target_unit, rng, world);
        }

        for (id, unit_id, killer) in battle.deaths {
            let cause_of_death = CauseOfDeath::KilledInBattle(killer);
            kill_creature(world, id, unit_id, *target, cause_of_death, resources);
        }

        for (id, xp) in battle.xp_add {
            let mut creature = world.creatures.get_mut(&id);
            creature.experience += xp;
        }
    }
}

// Plot stuff

pub(crate) fn start_plot(world: &mut World, creature_id: CreatureId, goal: PlotGoal) {
    let plot = Plot::new(goal, creature_id, world);
    let plot_id = world.plots.add(plot);
    history_trace!("plot_start creature_id:{:?} plot_id:{:?}", creature_id, plot_id);

    let mut creature = world.creatures.get_mut(&creature_id);
    creature.supports_plot = Some(plot_id);

}

pub(crate) fn find_supporters_for_plot(world: &mut World, creature_id: CreatureId) {
    let creature = world.creatures.get(&creature_id);
    let plot_id_o = creature.supports_plot;
    // TODO(IhlgIYVA): Kind of a smell
    if plot_id_o.is_none() {
        return;
    }
    let plot_id = plot_id_o.expect("Shouldn't happen");
    let mut plot = world.plots.get_mut(&plot_id);

    for relationship in creature.relationships.iter() {

        if relationship.rival_or_worse() {
            continue;
        }

        // TODO(IhlgIYVA): Check if duplicate
        let mut relation = world.creatures.get_mut(&relationship.creature_id);

        if relation.supports_plot.is_some() || relation.death.is_some() {
            continue;
        }

        let mut can_support = false;

        // If shares a goal
        for goal in relation.goals.iter() {
            let plot_goal = goal.as_plot_goal();
            if let Some(plot_goal) = plot_goal {
                if plot_goal == plot.goal {
                    can_support = true;
                    break;

                }
            }
        }

        if can_support {
            history_trace!("plot_new_supporter creature_id:{:?} plot_id:{:?}", relationship.creature_id, plot_id);
            plot.add_supporter(plot_id, relationship.creature_id, &mut relation);
        }

    }

}

pub(crate) fn execute_plot(world: &mut World, unit_id: UnitId, creature_id: CreatureId, rng: &mut Rng, resources: &mut Resources) {
    let creature = world.creatures.get(&creature_id);
    let plot_id_o = creature.supports_plot;
    // TODO(IhlgIYVA): Kind of a smell
    if plot_id_o.is_none() {
        return;
    }
    let plot_id = plot_id_o.expect("Shouldn't happen");
    let plot = world.plots.get(&plot_id);
    history_trace!("execute_plot creature_id:{:?} plot_id:{:?} plot:{:?}", creature_id, plot_id, plot);

    let goal = plot.goal.clone();
    drop(creature);
    drop(plot);

    match goal {
        PlotGoal::KillBeast(target_id) => {

            // TODO(IhlgIYVA): Bug
            let creature = world.creatures.get(&target_id);
            if creature.death.is_some() {
                println!("plot: creature is already dead");
                return;
            }
            drop(creature);

            // TODO(IhlgIYVA): Performance for Unit
            let ret = world.units.iter_id_val::<UnitId>().find(|(_id, unit)| unit.borrow().creatures.contains(&target_id));
            if let Some((target_id, _)) = ret {

                // TODO(IhlgIYVA): Dupped code
                // TODO(IhlgIYVA): Separate plotters from unit
                // TODO(IhlgIYVA): Die outside of unit

                let battle;
                {
                    let unit = world.units.get(&unit_id);
                    let target_unit = world.units.get(&target_id);
                    battle = BattleSimulator::simulate_attack(unit_id, &unit, target_id, &target_unit, rng, world);
                }
        
                for (id, unit_id, killer) in battle.deaths {
                    let cause_of_death = CauseOfDeath::KilledInBattle(killer);
                    kill_creature(world, id, unit_id, target_id, cause_of_death, resources);
                }
        
                for (id, xp) in battle.xp_add {
                    let mut creature = world.creatures.get_mut(&id);
                    creature.experience += xp;
                }

            } else {
                // TODO(IhlgIYVA): Error handling
                println!("<plot> How????");
            }

        }
    }

    let mut plot = world.plots.get_mut(&plot_id);
    plot.verify_success(&world);

}


// Global functions

pub(crate) fn kill_creature(world: &mut World, creature_id: CreatureId, unit_from_id: UnitId, unit_death_id: UnitId, cause_of_death: CauseOfDeath, resources: &mut Resources) {
    let now = world.date.clone();
    let died_home = unit_from_id == unit_death_id;
    {
        let mut creature = world.creatures.get_mut(&creature_id);
        if creature.death.is_some() {
            warn!("Trying to kill already dead creature");
            return;
        }
        creature.death = Some((now.clone(), cause_of_death));
        if let Some(spouse_id) = creature.spouse {
            let mut spouse = world.creatures.get_mut(&spouse_id);
            spouse.spouse = None;
        }
        let mut unit = world.units.get_mut(&unit_from_id);
        let i = unit.creatures.iter().position(|id| *id == creature_id).unwrap();
        let id = unit.creatures.remove(i);

        if let Some(plot_id) = creature.supports_plot {
            let mut plot = world.plots.get_mut(&plot_id);
            plot.remove_supporter(creature_id, &mut creature);
        }

        // Else, the body is lost
        if died_home {
            unit.cemetery.push(id);
        } else {
            let mut death_unit = world.units.get_mut(&unit_death_id);
            if let Some(settlement) = &mut death_unit.settlement {
                let species = resources.species.get(&creature.species);
                for drop in species.drops.iter() {
                    settlement.add_material(drop, 1);
                }
            }
        }    

        drop(unit);

        let mut inheritor = None;
        let mut has_possession = false;

        if let Some(details) = &creature.details {
            if details.inventory.len() > 0 {
                has_possession = true;
                if died_home {
                    for candidate_id in creature.offspring.iter() {
                        let candidate = world.creatures.get(candidate_id);
                        if candidate.death.is_none() {
                            inheritor = Some(*candidate_id);
                            break;
                        }
                    }
                }
            }
        }

        // TODO(IhlgIYVA): Extract
        if let CauseOfDeath::KilledInBattle(killer_id) = &cause_of_death {
            for relationship in creature.relationships.iter() {
                let relationship_creature_id = relationship.creature_id;
                let mut relationship_creature = world.creatures.get_mut(&relationship_creature_id);
                let relationship = relationship_creature.relationship_find(creature_id);
                if let Some(relationship) = relationship {
                    if relationship.friend_or_better() {
                        // TODO(IhlgIYVA): How did I get to a point where it killed his own "friend"?
                        if relationship_creature_id == *killer_id {
                            println!("rel: {:?} killer: {:?}", relationship_creature_id, killer_id);
                            continue
                        }
                        let killer = world.creatures.get(killer_id);
                        let killer_relationship = relationship_creature.relationship_find_mut_or_insert(&relationship_creature_id, *killer_id, &killer);
                        killer_relationship.add_opinion(-75);

                        if killer_relationship.mortal_enemy_or_worse() {
                            // TODO(IhlgIYVA): Determinate
                            // TODO(IhlgIYVA): Magic number
                            if Rng::rand().rand_chance(0.8) {
                                let goal = Goal::KillBeast(*killer_id);
                                history_trace!("creature_add_goal creature_id:{:?} goal:{:?}", relationship_creature_id, goal);
                                relationship_creature.goals.push(goal);
                            }
                        }

                    }
                }
            }
        }

        // Purges unnecessary data after death
        creature.relationships.clear();

        drop(creature);

        if has_possession {
            if let Some(inheritor_id) = inheritor {
                transfer_inventory(creature_id, inheritor_id, world);
            } else {
                if died_home {
                    let creature = world.creatures.get(&creature_id);
                    if let Some(details) = &creature.details {
                        let inventory = details.inventory.clone();
                        world.events.push(Event::BurriedWithPosessions { date: now.clone(), creature_id, items_ids: inventory });
                    }
                } else {
                    drop_inventory(creature_id, world);
                }
            }
        }

        // TODO: Inherit leadership

    }
    world.events.push(Event::CreatureDeath { date: now.clone(), creature_id: creature_id, cause_of_death: cause_of_death });
}

// Artifact operations


pub(crate) fn add_item_to_inventory(item_id: ItemId, item: &mut Item, new_owner_id: CreatureId, new_owner: &mut Creature) {
    new_owner.details().inventory.push(item_id);
    item.owner = Some(new_owner_id);
}


fn transfer_inventory(current_id: CreatureId, new_owner_id: CreatureId, world: &mut World) {
    let mut current = world.creatures.get_mut(&current_id);
    let mut inventory: Vec<ItemId> = current.details().inventory.drain(..).collect();
    for item_id in inventory.iter() {
        let mut item = world.artifacts.get_mut(item_id);
        item.owner = Some(new_owner_id);
    }

    history_trace!("transfer_inventory {:?}", current_id, new_owner_id);

    let mut new_owner = world.creatures.get_mut(&new_owner_id);
    new_owner.details().inventory.append(&mut inventory);
    for item in inventory.iter() {
        world.events.push(Event::InheritedArtifact { date: world.date.clone(), creature_id: new_owner_id, from: current_id, item: *item });
    }
}

fn drop_inventory(creature_id: CreatureId, world: &mut World) {
    history_trace!("drop_inventory {:?}", creature_id);

    let mut current = world.creatures.get_mut(&creature_id);
    let inventory: Vec<ItemId> = current.details().inventory.drain(..).collect();
    for item_id in inventory.iter() {
        let mut item = world.artifacts.get_mut(item_id);
        item.owner = None;
    }

    // TODO(NJ5nTVIV): Add to death unit
}