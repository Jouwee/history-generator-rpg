use graphics::{ellipse, image, rectangle, Context, Text};
use opengl_graphics::{GlGraphics, GlyphCache, Texture};
use piston::RenderArgs;
use crate::graphics::Transformed;

use super::{assets::OldAssets, Color};

pub struct RenderContext<'a, 'b> {
    pub args: &'a RenderArgs,
    pub original_transform: [[f64; 3]; 2],
    pub context: Context,
    pub gl: &'a mut GlGraphics,
    pub assets: &'b mut OldAssets,
    pub layout_rect: [f64; 4],
    pub camera_rect: [f64; 4],
    pub transform_queue: Vec<[[f64; 3]; 2]>,
    // TODO: Repo
    pub default_font: &'b mut GlyphCache<'b>,
    pub small_font: &'b mut GlyphCache<'b>,
    pub textures: Vec<Texture>
}

impl<'a, 'b> RenderContext<'a, 'b> {

    pub fn pixel_art(&mut self, s: u8) {
        self.scale(s as f64);
    }

    pub fn scale(&mut self, s: f64) {
        let s = s as f64;
        self.context.transform = self.context.transform.scale(s, s);
        self.layout_rect[2] = self.layout_rect[2] / s;
        self.layout_rect[3] = self.layout_rect[3] / s;
        self.camera_rect[2] = self.camera_rect[2] / s;
        self.camera_rect[3] = self.camera_rect[3] / s;
    }

    pub fn translate(&mut self, x: f64, y: f64) {
        self.context.transform = self.context.transform.trans(x, y);
    }

    pub fn rotate90(&mut self) {
        self.context.transform = self.context.transform.rot_deg(90.);
    }

    pub fn center_camera_on(&mut self, pos: [f64; 2]) {
        self.camera_rect[0] = (pos[0] - self.camera_rect[2] / 2.).round();
        self.camera_rect[1] = (pos[1] - self.camera_rect[3] / 2.).round();
        self.context.transform = self.context.transform.trans(-self.camera_rect[0], -self.camera_rect[1]);
    }

    pub fn push(&mut self) {
        self.transform_queue.push(self.context.transform);
    }

    pub fn try_pop(&mut self) -> Result<(), ()> {
        self.context.transform = self.transform_queue.pop().ok_or(())?;
        return Ok(())
    }

    pub fn rectangle_fill(&mut self, rect: [f64; 4], color: Color) {
        rectangle(color.f32_arr(), rect, self.context.transform, self.gl);
    }

    pub fn circle(&mut self, rect: [f64; 4], color: Color) {
        ellipse(color.f32_arr(), rect, self.context.transform, self.gl);
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

    //pub fn text(&mut self, text: &str, font: &mut GlyphCache, font_size: u32, position: [f64; 2], color: Color) {
    pub fn text_small(&mut self, text: &str, font_size: u32, position: [f64; 2], color: Color) {
        Text::new_color(color.f32_arr(), font_size)
            .round()
            .draw_pos(
                text,
                [position[0], position[1]],
                self.small_font,
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

    pub fn texture(&mut self, texture: Texture, position: [f64; 2]) {
        let transform = self.context.transform.trans(position[0], position[1]);
        // Workaround for a behaviour of piston where it passes a reference of the texture to the backend for async rendering,
        // but that texture might be dropped before the frame is rendered, resulting in a black square.
        // So I save the texture in this array, which is dropped after the frame is rendered.
        self.textures.push(texture);
        image(self.textures.last().unwrap(), transform, self.gl);
    }

    pub fn texture_ref(&mut self, texture: &Texture, position: [f64; 2]) {
        let transform = self.context.transform.trans(position[0], position[1]);
        image(texture, transform, self.gl);
    }

    pub fn spritesheet(&mut self, texture_name: &str, sprite: (u32, u32), position: [f64; 2]) {
        let spritesheet = self.assets.spritesheet(texture_name, (16, 16));
        let transform = self.context.transform.trans(position[0], position[1]);
        image(spritesheet.sprite(sprite.0, sprite.1), transform, self.gl);
    }

}