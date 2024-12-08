use graphics::{image, rectangle, Context, Text};
use opengl_graphics::{GlGraphics, GlyphCache};
use piston::RenderArgs;
use crate::graphics::Transformed;

use super::{assets::Assets, Color};

pub struct RenderContext<'a, 'b> {
    pub args: &'a RenderArgs,
    pub original_transform: [[f64; 3]; 2],
    pub context: Context,
    pub gl: &'a mut GlGraphics,
    pub assets: &'b mut Assets,
    // TODO: Repo
    pub default_font: &'b mut GlyphCache<'b>
}

impl<'a, 'b> RenderContext<'a, 'b> {
    pub fn rectangle_fill(&mut self, rect: [f64; 4], color: Color) {
        rectangle(color.f32_arr(), rect, self.context.transform, self.gl);
    }

    //pub fn text(&mut self, text: &str, font: &mut GlyphCache, font_size: u32, position: [f64; 2], color: Color) {
    pub fn text(&mut self, text: &str, font_size: u32, position: [f64; 2], color: Color) {
        Text::new_color(color.f32_arr(), font_size)
            .round()
            .draw_pos(
                text,
                [position[0], position[1]],
                self.default_font,
                &self.context.draw_state,
                self.context.transform,
                self.gl,
            )
            .unwrap();
    }

    pub fn image(&mut self, texture_name: &str, position: [f64; 2]) {
        let texture = self.assets.texture(texture_name);
        let transform = self.context.transform.trans(position[0], position[1]);
        image(texture, transform, self.gl);
    }

    pub fn spritesheet(&mut self, texture_name: &str, sprite: (u32, u32), position: [f64; 2]) {
        let spritesheet = self.assets.spritesheet(texture_name, (16, 16));
        let transform = self.context.transform.trans(position[0], position[1]);
        image(spritesheet.sprite(sprite.0, sprite.1), transform, self.gl);
    }

}