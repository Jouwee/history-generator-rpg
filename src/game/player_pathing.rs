use crate::{game::{chunk::{Chunk, PLAYER_IDX}, game_log::GameLog}, resources::action::ActionRunner, world::world::World, Actor, Coord2, GameContext, RenderContext, Update};

use super::TurnMode;

pub(crate) struct PlayerPathing {
    last_path_to: Option<Coord2>,
    preview: Option<Vec<Coord2>>,
    running: Option<Vec<Coord2>>,
    wait: f64,
}

impl PlayerPathing {
    
    pub(crate) fn new() -> Self {
        Self { 
            last_path_to: None,
            preview: None,
            running: None,
            wait: 0.,
        }
    }

    pub(crate) fn render(&mut self, turn_mode: &TurnMode, player: &Actor, ctx: &mut RenderContext) {
        let mut running = false;
        if let Some(path) = &self.running {
            for tile in path.iter() {
                ctx.image("gui/path.png", [tile.x * 24, tile.y * 24]);
                running = true;
            }
        }
        if !running {
            let mut remaining_ap = player.ap.action_points;
            if let Some(path) = &self.preview {
                for tile in path.iter().rev() {
                    if *turn_mode == TurnMode::RealTime || remaining_ap >= 0 {
                        ctx.image("gui/path.png", [tile.x * 24, tile.y * 24]);
                    } else {
                        ctx.image("gui/path.png", [tile.x * 24, tile.y * 24]);
                        // ctx.rectangle_fill([tile.x as f64 * 24. + 8., tile.y as f64 * 24. + 8., 8., 8.], COLOR_WHITE30"));
                    }
                    // TODO (OLaU4Dth): 
                    remaining_ap -= 20;
                }
            }
        }
    }

    pub(crate) fn should_recompute_pathing(&mut self, cursor: Coord2) -> bool {
        return match &self.last_path_to {
            None => true,
            Some(coord) => *coord != cursor,
        };
    }

    pub(crate) fn set_preview(&mut self, cursor: Coord2, mut path: Vec<Coord2>) {
        // Removes the first move as it's always the current position
        let _ = path.pop();
        self.preview = Some(path);
        self.last_path_to = Some(cursor);
    }

    pub(crate) fn get_preview_ap_cost(&self) -> i32 {
        if !self.is_running() {
            if let Some(preview) = &self.preview {
                // TODO(OLaU4Dth):
                return preview.len() as i32 * 20
            }
        }
        return 0
    }

    pub(crate) fn is_running(&self) -> bool {
        return self.running.is_some()
    }

    pub(crate) fn update_running(&mut self, chunk: &mut Chunk, world: &mut World, game_log: &mut GameLog, update: &Update, action_runner: &mut ActionRunner, ctx: &mut GameContext) {
        if self.add_update_delta(update.delta_time) {
            let pos = self.pop_running();
            if let Some(pos) = pos {
                let action_id = ctx.resources.actions.id_of("act:move");  
                let action = ctx.resources.actions.get(&action_id);  
                let result = action_runner.try_use(&action_id, action, PLAYER_IDX, pos, chunk, world, game_log, ctx);
                if result.is_err() {
                    self.clear_running();
                }
            }
        }
    }

    pub(crate) fn add_update_delta(&mut self, delta: f64) -> bool {
        self.wait += delta;
        // TODO(OLaU4Dth): Gamespeed option
        if self.wait >= 0.2 {
            self.wait -= 0.2;
            return true
        }
        return false
    }

    pub(crate) fn pop_running(&mut self) -> Option<Coord2> {
        if let Some(path) = &mut self.running {
            return path.pop()
        }
        return None
    }

    pub(crate) fn get_preview(&self) -> &Option<Vec<Coord2>> {
        return &self.preview;
    }

    pub(crate) fn start_running(&mut self, path: Vec<Coord2>) {
        self.running = Some(path.clone());
    }

    pub(crate) fn clear_running(&mut self) {
        self.running = None;
    }

}