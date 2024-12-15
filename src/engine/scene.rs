use crate::game::InputEvent;
use super::render::RenderContext;

pub struct Update {
    pub delta_time: f64,
    pub max_update_time: f64,
    pub updates_per_second: u32
}

pub trait Scene {
    fn render(&mut self, ctx: &mut RenderContext);
    fn update(&mut self, update: &Update);
    fn input(&mut self, evt: &InputEvent);
    fn cursor_move(&mut self, _pos: [f64; 2]) {}
}
