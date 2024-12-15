use crate::game::InputEvent;
use super::render::RenderContext;

pub mod button;
pub mod container;
pub mod dialog;
pub mod label;

pub trait GUINode {
    fn render(&mut self, _ctx: &mut RenderContext) {}
    fn update(&mut self) {}
    fn input(&mut self, _evt: &InputEvent) {}
    fn cursor_move(&mut self, _pos: [f64; 2]) {}

    fn compute_position(&self, position: &Position, parent_rect: [f64; 4], size: [f64; 2]) -> [f64; 2] {
        match position {
            Position::Anchored(Anchor::TopLeft, x, y) => [parent_rect[0] + *x, parent_rect[1] + *y],
            Position::Anchored(Anchor::BottomLeft, x, y) => [parent_rect[0] + *x, parent_rect[1] + parent_rect[3] - *y],
            Position::Anchored(Anchor::BottomRight, x, y) => [parent_rect[0] + parent_rect[2] - *x, parent_rect[1] + parent_rect[3] - *y],
            Position::Centered => [parent_rect[0] + parent_rect[2] / 2. - size[0] / 2., parent_rect[1] + parent_rect[3] / 2. - size[1] / 2.]
        }
    }

    fn parent_rect(&self, ctx: &RenderContext) -> [f64; 4] {
        ctx.layout_rect
    }

}

pub enum Position {
    Anchored(Anchor, f64, f64),
    Centered
}

pub enum Anchor {
    TopLeft,
    BottomLeft,
    BottomRight
}