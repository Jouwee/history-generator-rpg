use crate::{commons::{id_vec::Identified, rng::Rng}, history_trace, warn, world::{creature::{CauseOfDeath, Creature, CreatureId, Profession}, date::{Duration, WorldDate}, history_sim::{battle_simulator::BattleSimulator, storyteller::SiteChances}, item::{Item, ItemId}, plot::{Plot, PlotGoal, PlotId}, site::{Site, SiteId, SiteType}, world::World}};

pub(crate) struct CreatureSimulation {}

#[derive(Debug)]
pub(crate) enum CreatureSideEffect {
    None,
    Death(CauseOfDeath),
    HaveChild,
    LookForMarriage,
    LookForNewJob,
    MakeArtifact,
    BecomeBandit,
    AttackNearbySites,
    StartPlot(PlotGoal),
    FindSupportersForPlot,
    ExecutePlot,
    MoveOutToNewHouse,
}

impl CreatureSimulation {

    pub(crate) fn simulate_step_creature(_step: &Duration, now: &WorldDate, rng: &mut Rng, site: &Site, creature_id: &CreatureId, creature: &Creature, supported_plot: Option<Identified<PlotId, Plot>>, chances: &SiteChances) -> CreatureSideEffect {
        let age = (*now - creature.birth).get_years();
        // Death by disease
        if rng.rand_chance(chances.disease_death) {
            return CreatureSideEffect::Death(CauseOfDeath::Disease);
        }

        if creature.sim_flag_is_great_beast() {
            if rng.rand_chance(chances.great_beast_hunt) {
                return CreatureSideEffect::AttackNearbySites;
            }
        }

        // Get a profession
        if creature.sim_flag_is_inteligent() && site.site_type == SiteType::Village {

            match supported_plot {
                None => {
                    for goal in creature.goals.iter() {
                        let plot = goal.as_plot_goal();
                        if let Some(plot) = plot {
                            if rng.rand_chance(chances.start_plot) {
                                return CreatureSideEffect::StartPlot(plot);
                            }
                        }
                    }
                },
                Some(plot) => {
                    if rng.rand_chance(chances.work_on_plot) {
                        if rng.rand_chance(1. - plot.success_chance()) {
                            return CreatureSideEffect::FindSupportersForPlot;
                        } else {
                            return CreatureSideEffect::ExecutePlot;
                        }
                    }
                    
                }
            }

            if age >= 14 && creature.profession == Profession::None {
                return CreatureSideEffect::LookForNewJob;
            }
            if age >= 18 {

                // TODO: Not leader
                // TODO: Slow
                let home = site.structure_occupied_by(creature_id);
                if let Some(home) = home {
                    if home.occupant_count() > 5 {
                        return CreatureSideEffect::MoveOutToNewHouse;
                    }
                }

                // Have child
                if creature.gender.is_female() && creature.spouse.is_some()  {
                    if rng.rand_chance(Self::chance_of_child(now, creature, chances)) {
                        return CreatureSideEffect::HaveChild;
                    }
                }
                // Find a spouse
                if creature.spouse.is_none() {
                    if rng.rand_chance(chances.marry) {
                        return CreatureSideEffect::LookForMarriage;
                    }
                }

                // Look for new job
                if !creature.profession.is_for_life() {
                    if rng.rand_chance(chances.change_job) {
                        return CreatureSideEffect::LookForNewJob;
                    }
                    if creature.spouse.is_none() && rng.rand_chance(chances.leave_for_bandits) {
                        return CreatureSideEffect::BecomeBandit;
                    }
                }
                
            }
        }

        if age >= 40 {
            // Death of old age
            if rng.rand_chance(chances.base_multiplier * Self::chance_of_death_by_old_age(age as f32)) {
                return CreatureSideEffect::Death(CauseOfDeath::OldAge);
            }
        }

        match creature.profession {
            Profession::Blacksmith | Profession::Sculptor => {
                if rng.rand_chance(chances.make_inspired_artifact) {
                    return CreatureSideEffect::MakeArtifact;
                }
            },
            _ => ()
        }
        return CreatureSideEffect::None
    }

    fn chance_of_child(now: &WorldDate, creature: &Creature, chances: &SiteChances) -> f32 {
        let age = (*now - creature.birth).get_years() as f32;        
        let fertility_mult = (0.96 as f32).powf(age - 18.) * (0.92 as f32).powf(age - 18.);

        return (chances.have_child * fertility_mult).clamp(0., 1.);
    }

    fn chance_of_death_by_old_age(age: f32) -> f32 {
        return ((age - 40.) / 60.).clamp(0., 1.)
    }

}

// Legendary beasts
const HUNT_RADIUS_SQRD: f32 = 5.*5.;

pub(crate) fn attack_nearby_site(world: &mut World, rng: &mut Rng, site_id: SiteId) {
    let mut candidates = Vec::new();
    {
        let source_site = world.sites.get(&site_id);
        for (id, site) in world.sites.iter_id_val::<SiteId>() {
            if id != site_id {
                let site = site.borrow();
                if site.creatures.len() > 0 && site.xy.dist_squared(&source_site.xy) < HUNT_RADIUS_SQRD {
                    candidates.push(id);
                    break;
                }
            }
        }
    }

    if let Some(target) = rng.item(&candidates) {
        let battle;
        {
            let site = world.sites.get(&site_id);
            let target_site = world.sites.get(target);
            battle = BattleSimulator::simulate_attack(site_id, &site, *target, &target_site, rng, world);
        }

        for (id, site_id, killer_id) in battle.deaths {
            let killer = world.creatures.get(&killer_id);
            let item_used = match &killer.details {
                Some(details) => details.inventory.first().and_then(|id| Some(*id)),
                None => None
            };
            let cause_of_death = CauseOfDeath::KilledInBattle(killer_id, item_used);
            drop(killer);
            world.kill_creature(id, site_id, *target, cause_of_death);
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

pub(crate) fn execute_plot(world: &mut World, site_id: SiteId, creature_id: CreatureId, rng: &mut Rng) {
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
                warn!("plot: creature is already dead");
                return;
            }
            drop(creature);

            // TODO(IhlgIYVA): Performance for Site
            let ret = world.sites.iter_id_val::<SiteId>().find(|(_id, site)| site.borrow().creatures.contains(&target_id));
            if let Some((target_id, _)) = ret {

                // TODO(IhlgIYVA): Dupped code
                // TODO(IhlgIYVA): Separate plotters from site
                // TODO(IhlgIYVA): Die outside of site

                let battle;
                {
                    let site = world.sites.get(&site_id);
                    let target_site = world.sites.get(&target_id);
                    battle = BattleSimulator::simulate_attack(site_id, &site, target_id, &target_site, rng, world);
                }
        
                for (id, site_id, killer_id) in battle.deaths {
                    let killer = world.creatures.get(&killer_id);
                    let item_used = match &killer.details {
                        Some(details) => details.inventory.first().and_then(|id| Some(*id)),
                        None => None
                    };
                    let cause_of_death = CauseOfDeath::KilledInBattle(killer_id, item_used);
                    drop(killer);
                    world.kill_creature(id, site_id, target_id, cause_of_death);
                }
        
                for (id, xp) in battle.xp_add {
                    let mut creature = world.creatures.get_mut(&id);
                    creature.experience += xp;
                }

            } else {
                // TODO(IhlgIYVA): Error handling
                warn!("[plot] Shouldn't happen");
            }

        }
    }

    let mut plot = world.plots.get_mut(&plot_id);
    plot.verify_success(&world);

}

// Artifact operations


pub(crate) fn add_item_to_inventory(item_id: ItemId, item: &mut Item, new_owner_id: CreatureId, new_owner: &mut Creature) {
    new_owner.details().inventory.push(item_id);
    item.owner = Some(new_owner_id);
}