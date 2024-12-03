use graphics::{image, rectangle, Context, Text};
use opengl_graphics::{GlGraphics, GlyphCache, Texture};
use piston::RenderArgs;
use crate::graphics::Transformed;

use super::Color;

pub struct RenderContext<'a, 'b> {
    pub args: &'a RenderArgs,
    pub context: Context,
    pub gl: &'a mut GlGraphics,
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
            .draw(
                text,
                self.default_font,
                &self.context.draw_state,
                self.context.transform.trans(position[0], position[1]),
                self.gl,
            )
            .unwrap();
    }

    pub fn image(&mut self, texture: &Texture, position: [f64; 2]) {
        let transform = self.context.transform.trans(position[0], position[1]);
        image(texture, transform, self.gl);
    }
}