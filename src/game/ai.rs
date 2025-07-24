use std::{collections::VecDeque, time::Instant, vec};

use crate::{commons::{astar::{AStar, MovementCost}, bitmask::bitmask_get}, engine::geometry::Coord2, game::actor::actor::ActorType, resources::action::{Action, ActionId, Actions, Affliction, ActionEffect, ActionTarget, FILTER_CAN_OCCUPY, FILTER_CAN_VIEW}, GameContext};

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

    pub(crate) fn next_action<'a>(&mut self, actions: &'a Actions) -> Option<(ActionId, &'a Action, Coord2)> {
        let action = self.actions.pop_front();
        if let Some((action_id, pos)) = action {
            let action = actions.get(&action_id);
            self.delay = 0.;
            self.delay_target = action.ap_cost as f64 / 200.;
            return Some((action_id, action, pos))
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
            actions.id_of("act:move"),
            // actions.id_of("act:move_down"),
            // actions.id_of("act:move_left"),
            // actions.id_of("act:move_up"),
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

        // println!("all paths:");
        // for path in results.iter() {
        //     println!("{:?}", path)
        // }

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
            let points_to_check = match &action.target {
                ActionTarget::Caster => vec!(ctx.xy),
                // TODO(REUw3poo): implement
                ActionTarget::Actor { range, filter_mask } => {
                    let range= *range as i32;
                    let range_s = (range * range) as f32;
                    let mut points = Vec::new();
                    for x in ctx.xy.x-range..(ctx.xy.x+range+1) {
                        for y in ctx.xy.y-range..(ctx.xy.y+range+1) {
                            let p = Coord2::xy(x, y);
                            // TODO: Dupped
                            if p == ctx.xy || ctx.xy.dist_squared(&p) > range_s {
                                continue
                            }
                            if bitmask_get(*filter_mask, FILTER_CAN_VIEW) {
                                if !chunk.map.check_line_of_sight(&ctx.xy, &p) {
                                    continue
                                }
                            }
                            points.push(p);
                        }
                    }
                    points
                },
                ActionTarget::Tile { range, filter_mask } => {
                    let range= *range as i32;
                    let range_s = (range * range) as f32;
                    let mut points = Vec::new();
                    for x in ctx.xy.x-range..(ctx.xy.x+range+1) {
                        for y in ctx.xy.y-range..(ctx.xy.y+range+1) {
                            let p = Coord2::xy(x, y);
                            // TODO: Dupped
                            if p == ctx.xy || ctx.xy.dist_squared(&p) > range_s {
                                continue
                            }
                            if bitmask_get(*filter_mask, FILTER_CAN_OCCUPY) {
                                if chunk.map.blocks_movement(p) {
                                    continue
                                }
                            }
                            points.push(p);
                        }
                    }
                    points
                },
            };
            for point in points_to_check {
                let mut ctx = ctx.clone();
                ctx.ap -= action.ap_cost as i32;
                ctx.stamina -= action.stamina_cost;
                ctx.depth += 1;
                ctx.actions.push((*action_id, point));

            
                for effect in action.effects.iter() {
                    match effect {
                        ActionEffect::Damage(damage_model) => {
                            for (_i, _actor) in action.area.filter(point, ctx.actor_idx, chunk.actors_iter()) {
                                // SMELL: Easy to forget
                                ctx.damage_score += (damage_model.bludgeoning + damage_model.slashing + damage_model.piercing + damage_model.arcane + damage_model.fire) as f64;
                            }
                        }
                        ActionEffect::Inflicts { affliction } => {
                            for (_i, _actor) in action.area.filter(point, ctx.actor_idx, chunk.actors_iter()) {
                                let score = match affliction {
                                    Affliction::Bleeding { duration } => 1. * *duration as f64,
                                    Affliction::OnFire { duration } => 1. * *duration as f64,
                                    Affliction::Stunned { duration } => 0.8 * *duration as f64,
                                    Affliction::Poisoned { duration } => 0.8 * *duration as f64,
                                };
                                ctx.damage_score += score;
                            }
                        },
                        ActionEffect::ReplaceObject { tile: _ } => {
                            // TODO:
                        },
                        ActionEffect::TeleportActor => {
                            ctx.xy = point;
                            ctx.position_score = Self::compute_position_score(&ctx, astar, chunk);
                        },
                        ActionEffect::Inspect => {
                            // TODO:
                        },
                        ActionEffect::Dig => {
                            // TODO:
                        },
                        ActionEffect::Sleep => {
                            // TODO:
                        },
                        ActionEffect::PickUp => {
                            // TODO:
                        },
                    }
                }

                ctx.compute_final_score();
                paths += Self::sim_step(ctx, results, available_actions, astar, actions, chunk);
            }
        }
        return paths
    }

    fn compute_position_score(ctx: &SimContext, astar: &mut AStar, chunk: &Chunk) -> f64 {
        let dist = ctx.xy.dist(&chunk.player().xy) as f64;
        if dist <= 1.5 {
            return 1.;
        }        
        let path = astar.get_path(ctx.xy);
        if path.len() == 0 {
            return 0.
        }
        return 1. / path.len() as f64;
    }

    fn add_to_results(ctx: SimContext, results: &mut Vec<SimContext>) {
        // TODO: Binary search
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