use std::ops::ControlFlow;

use crate::{game::InputEvent, GameContext};
use super::render::RenderContext;

pub(crate) struct Update {
    pub(crate) delta_time: f64,
    pub(crate) max_update_time: f64,
    pub(crate) mouse_pos_cam: [f64; 2],
}

pub(crate) trait Scene {
    fn init(&mut self, _ctx: &mut GameContext) {}
    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext);
    fn update(&mut self, update: &Update, ctx: &mut GameContext);
    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<()>;
}
