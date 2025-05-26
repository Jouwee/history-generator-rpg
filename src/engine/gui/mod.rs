use crate::GameContext;
use super::{render::RenderContext, scene::Update};

pub(crate) mod button;
pub(crate) mod new_ui;
pub(crate) mod tooltip;

pub(crate) trait GUINode {
    fn render(&mut self, _ctx: &mut RenderContext, _game_ctx: &mut GameContext) {}
    fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {}

    fn compute_position(&self, position: &Position, parent_rect: [f64; 4], size: [f64; 2]) -> [f64; 2] {
        let p;
        match position {
            Position::Anchored(Anchor::TopRight, x, y) => p = [parent_rect[0] + parent_rect[2] - size[0] - *x, parent_rect[1] + *y],
        }
        return [p[0].round(), p[1].round()]
    }

    fn parent_rect(&self, ctx: &RenderContext) -> [f64; 4] {
        ctx.layout_rect
    }

    fn min_size(&self, _ctx: &mut RenderContext) -> [f64; 2] {
        [0., 0.]
    }

}

pub(crate) enum Position {
    Anchored(Anchor, f64, f64),
}

pub(crate) enum Anchor {
    TopRight,
}