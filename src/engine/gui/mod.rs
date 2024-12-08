use crate::game::InputEvent;
use super::render::RenderContext;

pub mod button;
pub mod label;

pub trait GUINode {
    fn render(&self, _ctx: &mut RenderContext) {}
    fn update(&mut self) {}
    fn input(&mut self, _evt: &InputEvent) {}
    fn cursor_move(&mut self, _pos: [f64; 2]) {}

    fn compute_position(&self, position: &Position) -> [f64; 2] {
        match position {
            Position::Anchored(Anchor::TopLeft, x, y) => [*x, *y]
        }
    }

}

pub enum Position {
    Anchored(Anchor, f64, f64)
}

pub enum Anchor {
    TopLeft
}