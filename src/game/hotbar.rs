use graphics::CharacterCache;
use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::engine::{render::RenderContext, scene::Update, Color};

use super::{actor::Actor, InputEvent};

pub struct Hotbar {
    background: Texture
}

impl Hotbar {
    pub fn new() -> Hotbar {
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let background = ImageReader::open("assets/sprites/gui/hotbar/background.png").unwrap().decode().unwrap();
        Hotbar {
            background: Texture::from_image(&background.to_rgba8(), &settings)
        }
    }

}

impl<'a> NodeWithState<HotbarState<'a>> for Hotbar {
    fn render(&mut self, state: HotbarState, ctx: &mut RenderContext) {
        // Background
        let center = ctx.layout_rect[2] / 2.;
        let base_pos = [center - 128., ctx.layout_rect[3] - 34.];
        ctx.texture_ref(&self.background, base_pos);

        let mut hp_pos = base_pos.clone();
        hp_pos[0] = hp_pos[0] + 64.;
        hp_pos[1] = hp_pos[1] + 3.;

        let health_pct = (state.player.hp.health_points / state.player.hp.max_health_points as f32) as f64;
        ctx.rectangle_fill([hp_pos[0], hp_pos[1], (62. * health_pct).round(), 5.], Color::from_hex("994444"));

        let text = format!("{:.0}/{:.0}", state.player.hp.health_points, state.player.hp.max_health_points);
        let text_width = ctx.small_font.width(5, &text).unwrap_or(0.);
        ctx.text_small(&text, 5, [(hp_pos[0] + 31. - text_width / 2.).round(), hp_pos[1] + 5.], Color::from_hex("ffffff"));

        let mut ap_pos = base_pos.clone();
        ap_pos[0] = ap_pos[0] + 131.;
        ap_pos[1] = ap_pos[1] + 3.;

        let action_pct = (state.player.ap.action_points as f32 / state.player.ap.max_action_points as f32) as f64;
        ctx.rectangle_fill([ap_pos[0], ap_pos[1], (62. * action_pct).round(), 5.], Color::from_hex("446d99"));

        let text = format!("{:.0}/{:.0}", state.player.ap.action_points, state.player.ap.max_action_points);
        let text_width = ctx.small_font.width(5, &text).unwrap_or(0.);
        ctx.text_small(&text, 5, [(ap_pos[0] + 31. - text_width / 2.).round(), ap_pos[1] + 5.], Color::from_hex("ffffff"));

    }
    fn update(&mut self, _state: HotbarState, _update: &Update) {
        
    }
    fn input(&mut self, _state: HotbarState, _evt: &InputEvent) {
        
    }
}


pub struct HotbarState<'a> {
    player: &'a Actor
}


impl<'a> HotbarState<'a> {
    pub fn new(player: &'a Actor) -> HotbarState<'a> {
        HotbarState { player }
    }
}

pub trait NodeWithState<T> {
    fn render(&mut self, _state: T, _ctx: &mut RenderContext) {}
    fn update(&mut self, _state: T, _update: &Update) {}
    fn input(&mut self, _state: T, _evt: &InputEvent) {}
}