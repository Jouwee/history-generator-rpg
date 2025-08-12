use std::{collections::VecDeque, time::Instant, vec};

use crate::{commons::astar::{AStar, MovementCost}, engine::geometry::Coord2, game::chunk::AiGroups, info, resources::action::{Action, ActionEffect, ActionId, ActionTarget, Actions, Affliction}, GameContext};

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

#[derive(Clone)]
pub(crate) enum AiState {
    Disabled,
    Fight,
}

pub(crate) struct AiSolver {

}

const DETECTION_RANGE_SQRD: f32 = 12. * 12.;

impl AiSolver {

    pub(crate) fn check_state(actor: &Actor, chunk: &Chunk) -> AiState {
        match actor.ai_state {
            AiState::Disabled => {
                for hostile in chunk.actors_iter() {
                    if chunk.ai_groups.is_hostile(actor.ai_group, hostile.ai_group) && actor.xy.dist_squared(&hostile.xy) <= DETECTION_RANGE_SQRD {
                        return AiState::Fight;
                    }
                }
                return AiState::Disabled
            },
            AiState::Fight => AiState::Fight
        }
    }

    pub(crate) fn choose_actions(actions: &Actions, actor: &Actor, actor_idx: usize, chunk: &Chunk, ctx: &GameContext) -> AiRunner {

        if let AiState::Disabled = actor.ai_state {
            let mut runner = AiRunner::new();
            runner.actions = VecDeque::from(Vec::new());
            return runner
        }

        if !chunk.ai_groups.is_hostile(AiGroups::player(), actor.ai_group) {
            let mut runner = AiRunner::new();
            runner.actions = VecDeque::from(Vec::new());
            return runner
        }

        let now = Instant::now();

        let mut result = None;
        
        
        let mut all_actions = vec!(
            actions.id_of("act:move"),
        );
        let species = ctx.resources.species.get(&actor.species);
        all_actions.extend(species.innate_actions.iter());
        for (_slot, item) in actor.inventory.all_equipped() {
            if let Some(action_provider) = &item.action_provider {
                all_actions.extend(action_provider.actions.clone());
            }
        }

        // Removes actions on cooldown
        let all_actions = all_actions.iter()
            .map(|a| *a)
            .filter(|action_id| !actor.cooldowns.iter().any(|cooldown| cooldown.0 == *action_id))
            .collect();

        let ctx = SimContext {
            actor_idx,
            ai_group: actor.ai_group,
            actions: Vec::new(),
            xy: actor.xy,
            ap: actor.ap.action_points,
            stamina: actor.stamina.stamina,
            depth: 1,
            score: 0.,
            position_score: 0.,
            hostile_damage: 0.,
            team_damage: 0.,
        };

        let mut astar = AStar::new(chunk.size, chunk.player().xy);
        
        astar.find_path(ctx.xy, |xy| {
            if !chunk.size.in_bounds(xy) || !chunk.can_occupy(&xy) {
                return MovementCost::Impossible;
            } else {
                return MovementCost::Cost(1.);
            }
        });

        let paths = Self::sim_step(ctx, &mut result, &all_actions, &mut astar, actions, chunk);


        let mut runner = AiRunner::new();
        if let Some(path) = result {
            runner.actions = VecDeque::from(path.actions.clone());
            info!("Winner: {:?}", path)
        }
        let elapsed = now.elapsed();
        info!("AI checked {} paths, elapsed {:.2?}", paths, elapsed);

        return runner
    }

    fn sim_step(ctx: SimContext, result: &mut Option<SimContext>, available_actions: &Vec<ActionId>, astar: &mut AStar, actions: &Actions, chunk: &Chunk) -> u32 {
        Self::add_to_results(ctx.clone(), result);
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
                ActionTarget::Actor { range, filter_mask: _ } | ActionTarget::Tile { range, filter_mask: _ } => {
                    let range= *range as i32;
                    let mut points = Vec::new();
                    for x in ctx.xy.x-range..(ctx.xy.x+range+1) {
                        for y in ctx.xy.y-range..(ctx.xy.y+range+1) {
                            let p = Coord2::xy(x, y);
                            if action.target.can_use(&ctx.xy, chunk, &p).is_ok() {
                                points.push(p);
                            }
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
                        ActionEffect::Damage{ damage, add_weapon: _ } => {
                            for (_i, actor) in action.area.filter(point, ctx.actor_idx, chunk.actors_iter()) {
                                if chunk.ai_groups.is_hostile(ctx.ai_group, actor.ai_group) {
                                    ctx.hostile_damage += damage.average() as f64;
                                } else {
                                    ctx.team_damage += damage.average() as f64;
                                }
                            }
                        }
                        ActionEffect::Inflicts { affliction } => {
                            for (_i, actor) in action.area.filter(point, ctx.actor_idx, chunk.actors_iter()) {
                                let score = match affliction {
                                    Affliction::Bleeding { duration } => 5. * *duration as f64,
                                    Affliction::OnFire { duration } => 5. * *duration as f64,
                                    Affliction::Stunned { duration } => 4. * *duration as f64,
                                    Affliction::Poisoned { duration } => 5. * *duration as f64,
                                };
                                if chunk.ai_groups.is_hostile(ctx.ai_group, actor.ai_group) {
                                    ctx.hostile_damage += score;
                                } else {
                                    ctx.team_damage += score;
                                }
                            }
                        },
                        ActionEffect::ReplaceObject { tile: _ } => (),
                        ActionEffect::TeleportActor | ActionEffect::Walk => {
                            ctx.xy = point;
                            ctx.position_score = Self::compute_position_score(&ctx, astar, chunk);
                        },
                        ActionEffect::Inspect => (),
                        ActionEffect::Talk => (),
                        ActionEffect::Dig => (),
                        ActionEffect::Sleep => (),
                        ActionEffect::PickUp => (),
                    }
                }

                ctx.compute_final_score();
                paths += Self::sim_step(ctx, result, available_actions, astar, actions, chunk);
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

    fn add_to_results(ctx: SimContext, result: &mut Option<SimContext>) {
        let swap = match result {
            None => true,
            Some(r) => ctx.score > r.score
        };
        if swap {
            result.replace(ctx);
        }
    }

}

#[derive(Debug, Clone)]
struct SimContext {
    actor_idx: usize,
    ai_group: u8,
    actions: Vec<(ActionId, Coord2)>,
    xy: Coord2,
    ap: i32,
    stamina: f32,
    depth: u8,
    score: f64,
    position_score: f64,
    hostile_damage: f64,
    team_damage: f64,
}

impl SimContext {

    fn compute_final_score(&mut self) {
        // Tiny boost for simplicity (less actions), mostly to choose between ties
        let simplicity_boost = 0.01 / self.actions.len() as f64;
        self.score = self.position_score + (self.hostile_damage - self.team_damage) + simplicity_boost
    }

}