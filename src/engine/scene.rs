use crate::game::InputEvent;
use super::render::RenderContext;

pub trait Scene {
    fn render(&self, ctx: RenderContext);
    fn update(&mut self);
    fn input(&mut self, evt: &InputEvent);
    fn cursor_move(&mut self, _pos: [f64; 2]) {}
}
