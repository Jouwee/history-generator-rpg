use crate::GameContext;
use super::{render::RenderContext, scene::Update};

pub(crate) mod new_ui;
pub(crate) mod tooltip;

pub(crate) trait GUINode {
    fn render(&mut self, _ctx: &mut RenderContext, _game_ctx: &mut GameContext) {}
    fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {}
}