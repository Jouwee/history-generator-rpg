use crate::game::InputEvent;
use super::render::RenderContext;

pub mod label;

pub trait GUINode {
    fn render(&self, _ctx: &mut RenderContext) {}
    fn update(&mut self) {}
    fn input(&mut self, _evt: &InputEvent) {}

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