use crate::{game::InputEvent, GameContext};
use super::render::RenderContext;

pub struct Update {
    pub delta_time: f64,
    pub max_update_time: f64,
    pub updates_per_second: u32,
    pub mouse_pos_cam: [f64; 2],
    pub mouse_pos_gui: [f64; 2]
}

pub trait Scene {
    fn init(&mut self, _ctx: &mut GameContext) {}
    fn render(&mut self, ctx: &mut RenderContext);
    fn update(&mut self, update: &Update, ctx: &mut GameContext);
    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext);
    fn cursor_move(&mut self, _pos: [f64; 2]) {}
}
