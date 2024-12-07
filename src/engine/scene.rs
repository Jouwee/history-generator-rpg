use crate::game::InputEvent;
use super::render::RenderContext;

pub trait Scene {
    fn render(&self, ctx: RenderContext);
    fn update(&mut self);
    fn input(&mut self, evt: &InputEvent);
}
