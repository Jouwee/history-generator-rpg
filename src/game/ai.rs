use std::{collections::VecDeque, time::Instant, vec};

use crate::{commons::astar::{AStar, MovementCost}, engine::geometry::Coord2, game::actor::actor::ActorType, resources::action::{Action, ActionId, ActionType, Actions, Affliction, AfflictionChance, DamageType, SpellEffect, SpellTarget}, GameContext};

use super::{actor::actor::Actor, chunk::Chunk};

#[derive(Clone)]
pub(crate) struct AiRunner {
    pub(crate) actions: VecDeque<(ActionId, Coord2)>,
    pub(crate) delay: f64,
    pub(crate) delay_target: f64,
}

impl AiRunner {

    pub(crate) fn new() -> AiRunner {
        return AiRunner {
            actions: VecDeque::new(),
            delay: 0.,
            delay_target: 0.
        }
    }

    pub(crate) fn next_action<'a>(&mut self, actions: &'a Actions) -> Option<(&'a Action, Coord2)> {
        let action = self.actions.pop_front();
        if let Some((action, pos)) = action {
            let action = actions.get(&action);
            self.delay = 0.;
            self.delay_target = action.ap_cost as f64 / 200.;
            return Some((action, pos))
        }
        return None
    }

    pub(crate) fn waiting_delay(&mut self, delta: f64) -> bool {
        self.delay += delta;
        return self.delay <= self.delay_target;
    }

}

pub(crate) struct AiSolver {

}

impl AiSolver {

    pub(crate) fn choose_actions(actions: &Actions, actor: &Actor, actor_idx: usize, chunk: &Chunk, ctx: &GameContext) -> AiRunner {

        let now = Instant::now();

        let mut results = Vec::new();
        
        
        let mut all_actions = vec!(
            actions.id_of("act:move_right"),
            actions.id_of("act:move_down"),
            actions.id_of("act:move_left"),
            actions.id_of("act:move_up"),
        );
        let species = ctx.resources.species.get(&actor.species);
        all_actions.extend(species.innate_actions.iter());
        for (_slot, item) in actor.inventory.all_equipped() {
            if let Some(action_provider) = &item.action_provider {
                all_actions.extend(action_provider.actions.clone());
            }
        }

        let ctx = SimContext {
            actor_idx,
            actions: Vec::new(),
            xy: actor.xy,
            ap: actor.ap.action_points,
            stamina: actor.stamina.stamina,
            depth: 1,
            score: 0.,
            position_score: 0.,
            damage_score: 0.,
        };

        if actor.actor_type == ActorType::Passive {
            // println!("Actor is passive. skiping AI");
            let mut runner = AiRunner::new();
            runner.actions = VecDeque::from(ctx.actions.clone());
            return runner
        }

        let mut astar = AStar::new(chunk.size, chunk.player().xy);
        
        astar.find_path(ctx.xy, |xy| {
            if !chunk.size.in_bounds(xy) || chunk.map.blocks_movement(xy) {
                return MovementCost::Impossible;
            } else {
                return MovementCost::Cost(1.);
            }
        });

        let paths = Self::sim_step(ctx, &mut results, &all_actions, &mut astar, actions, chunk);


        let mut runner = AiRunner::new();
        if let Some(path) = results.first() {
            runner.actions = VecDeque::from(path.actions.clone());
            println!("Winner: {:?}", path)
        }
        let elapsed = now.elapsed();
        println!("AI checked {} paths, elapsed {:.2?}", paths, elapsed);
        return runner
    }

    fn sim_step(ctx: SimContext, results: &mut Vec<SimContext>, available_actions: &Vec<ActionId>, astar: &mut AStar, actions: &Actions, chunk: &Chunk) -> u32 {
        Self::add_to_results(ctx.clone(), results);
        if ctx.depth > 10 {
            return 1
        }
        let mut paths = 1;
        for action_id in available_actions {
            let action = actions.get(&action_id);
            if action.ap_cost as i32 > ctx.ap {
                continue;
            }
            if action.stamina_cost > ctx.stamina {
                continue;
            }
            match &action.action_type {
                ActionType::Move { offset } => {
                    let mut ctx = ctx.clone();
                    ctx.xy = ctx.xy + *offset;
                    if !chunk.size.in_bounds(ctx.xy) || chunk.map.blocks_movement(ctx.xy) {
                        continue;
                    }
                    ctx.ap -= action.ap_cost as i32;
                    ctx.stamina -= action.stamina_cost;
                    ctx.depth += 1;
                    ctx.actions.push((*action_id, ctx.xy));
                    ctx.position_score = Self::compute_position_score(&ctx, astar, chunk);
                    ctx.compute_final_score();
                    paths += Self::sim_step(ctx, results, available_actions, astar, actions, chunk);
                },
                ActionType::Targeted { damage, inflicts } => {
                    if ctx.xy.dist_squared(&chunk.player().xy) < 3. {
                        let mut ctx = ctx.clone();
                        ctx.ap -= action.ap_cost as i32;
                        ctx.stamina -= action.stamina_cost;
                        ctx.depth += 1;
                        ctx.actions.push((*action_id, chunk.player().xy));
                        if let Some(damage) = damage {
                            match &damage {
                                DamageType::Fixed(damage) => ctx.damage_score += (damage.bludgeoning + damage.piercing + damage.slashing) as f64,
                                DamageType::FromWeapon(damage) => ctx.damage_score += (damage.bludgeoning + damage.piercing + damage.slashing) as f64 * 2.,
                            }
                        }
                        if let Some(inflicts) = inflicts {
                            let score_mult = match inflicts.chance {
                                AfflictionChance::OnHit => 1.,
                            };
                            let score = match inflicts.affliction {
                                Affliction::Bleeding { duration } => 1. * duration as f64,
                                Affliction::Stunned { duration } => 0.8 * duration as f64,
                                Affliction::Poisoned { duration } => 0.8 * duration as f64,
                            };
                            ctx.damage_score += score * score_mult;
                        }
                        ctx.compute_final_score();
                        paths += Self::sim_step(ctx, results, available_actions, astar, actions, chunk);
                    }
                },
                ActionType::Spell { target, area, effect } => {
                    let points_to_check = match target {
                        SpellTarget::Actor => vec!(ctx.xy)
                    };
                    for point in points_to_check {
                        let mut ctx = ctx.clone();
                        ctx.ap -= action.ap_cost as i32;
                        ctx.stamina -= action.stamina_cost;
                        ctx.depth += 1;
                        ctx.actions.push((*action_id, point));

                        for (_i, _actor) in area.filter(point, ctx.actor_idx, chunk.actors_iter()) {
                            match effect {
                                SpellEffect::Inflicts { affliction } => {
                                    let score = match affliction {
                                        Affliction::Bleeding { duration } => 1. * *duration as f64,
                                        Affliction::Stunned { duration } => 0.8 * *duration as f64,
                                        Affliction::Poisoned { duration } => 0.8 * *duration as f64,
                                    };
                                    ctx.damage_score += score;
                                }
                            }
                        }

                        ctx.compute_final_score();
                        paths += Self::sim_step(ctx, results, available_actions, astar, actions, chunk);
                    }
                }
                _ => ()
            }
        }
        return paths
    }

    fn compute_position_score(ctx: &SimContext, astar: &mut AStar, chunk: &Chunk) -> f64 {
        let dist = ctx.xy.dist(&chunk.player().xy) as f64;
        if dist < 3. {
            return 0.;
        }        
        let path = astar.get_path(ctx.xy);
        if path.len() == 0 {
            return 0.
        }
        return 1. / path.len() as f64;
    }

    fn add_to_results(ctx: SimContext, results: &mut Vec<SimContext>) {
        let i = results.iter().enumerate().find(|(_i, c)| c.score < ctx.score);
        match i {
            None => {
                if results.len() < 10 {
                    results.push(ctx);
                }
            }
            Some((i, _c)) => {
                results.insert(i, ctx);
            }
        }
    }

}

#[derive(Debug, Clone)]
struct SimContext {
    actor_idx: usize,
    actions: Vec<(ActionId, Coord2)>,
    xy: Coord2,
    ap: i32,
    stamina: f32,
    depth: u8,
    score: f64,
    position_score: f64,
    damage_score: f64
}

impl SimContext {

    fn compute_final_score(&mut self) {
        // Tiny boost for simplicity (less actions), mostly to choose between ties
        let simplicity_boost = 0.01 / self.actions.len() as f64;
        self.score = self.position_score + self.damage_score + simplicity_boost
    }

}