use crate::{game::InputEvent, GameContext};
use super::{render::RenderContext, scene::Update};

pub mod button;
pub mod container;
pub mod dialog;
pub mod hlist;
pub mod label;
pub mod tooltip;
pub mod vlist;

pub trait GUINode {
    fn render(&mut self, _ctx: &mut RenderContext, _game_ctx: &mut GameContext) {}
    fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {}
    fn input(&mut self, _evt: &InputEvent, _ctx: &mut GameContext) {}

    fn compute_position(&self, position: &Position, parent_rect: [f64; 4], size: [f64; 2]) -> [f64; 2] {
        let p;
        match position {
            Position::Auto => p = [parent_rect[0], parent_rect[1]],
            Position::Anchored(Anchor::TopLeft, x, y) => p = [parent_rect[0] + *x, parent_rect[1] + *y],
            Position::Anchored(Anchor::BottomLeft, x, y) => p = [parent_rect[0] + *x, parent_rect[1] + parent_rect[3] - *y],
            Position::Anchored(Anchor::BottomRight, x, y) => p = [parent_rect[0] + parent_rect[2] - *x, parent_rect[1] + parent_rect[3] - *y],
            Position::Anchored(Anchor::BottomCenter, x, y) => p = [parent_rect[0] + (parent_rect[2] / 2. - size[0] / 2.) + *x, parent_rect[1] + parent_rect[3] + *y],
            Position::Centered => p = [parent_rect[0] + parent_rect[2] / 2. - size[0] / 2., parent_rect[1] + parent_rect[3] / 2. - size[1] / 2.]
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

pub enum Position {
    Auto,
    Anchored(Anchor, f64, f64),
    Centered
}

pub enum Anchor {
    TopLeft,
    BottomLeft,
    BottomRight,
    BottomCenter
}