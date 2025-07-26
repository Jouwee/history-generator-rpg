use graphics::{image, rectangle, Context, Text, Transformed};
use opengl_graphics::{GlGraphics, Texture};

use crate::engine::{asset::assets::Assets, assets::assets};

use super::{asset::{font::Font, image_sheet::ImageSheetAsset}, Color};

pub(crate) struct RenderContext<'a> {
    pub(crate) context: Context,
    pub(crate) gl: &'a mut GlGraphics,
    pub(crate) layout_rect: [f64; 4],
    pub(crate) camera_rect: [f64; 4],
    pub(crate) transform_queue: Vec<[[f64; 3]; 2]>,
    pub(crate) textures: Vec<Texture>,
    pub(crate) sprite_i: usize
}

impl<'a> RenderContext<'a> {

    pub(crate) fn pixel_art(&mut self, s: u8) {
        self.scale(s as f64);
    }

    pub(crate) fn scale(&mut self, s: f64) {
        let s = s as f64;
        self.context.transform = self.context.transform.scale(s, s);
        self.layout_rect[2] = self.layout_rect[2] / s;
        self.layout_rect[3] = self.layout_rect[3] / s;
        self.camera_rect[2] = self.camera_rect[2] / s;
        self.camera_rect[3] = self.camera_rect[3] / s;
    }

    pub(crate) fn center_camera_on(&mut self, pos: [f64; 2]) {
        self.camera_rect[0] = (pos[0] - self.camera_rect[2] / 2.).round();
        self.camera_rect[1] = (pos[1] - self.camera_rect[3] / 2.).round();
        self.context.transform = self.context.transform.trans(-self.camera_rect[0], -self.camera_rect[1]);
    }

    pub(crate) fn push(&mut self) {
        self.transform_queue.push(self.context.transform);
    }

    pub(crate) fn try_pop(&mut self) -> Result<(), ()> {
        self.context.transform = self.transform_queue.pop().ok_or(())?;
        return Ok(())
    }

    pub(crate) fn rectangle_fill(&mut self, rect: [f64; 4], color: Color) {
        rectangle(color.f32_arr(), rect, self.context.transform, self.gl);
    }

    pub(crate) fn text(&mut self, text: &str, font: &mut Font, position: [i32; 2], color: &Color) {
        Text::new_color(color.f32_arr(), font.size)
            .round()
            .draw_pos(
                text,
                [position[0] as f64, position[1] as f64],
                &mut font.glyphs,
                &self.context.draw_state,
                self.context.transform,
                self.gl,
            )
            .unwrap();
    }

    pub(crate) fn text_shadow(&mut self, text: &str, font: &mut Font, position: [i32; 2], color: &Color) {
        self.text(text, font, [position[0], position[1] + 1], &Color::rgb([0.; 3]));
        self.text(text, font, position, color);
    }

    pub(crate) fn image(&mut self, img: &str, position: [i32; 2], _assets: &mut Assets) {
        let img = assets().image(img);
        let transform = self.context.transform.trans(position[0] as f64, position[1] as f64);
        image(&img.texture, transform, self.gl);
    }

    pub(crate) fn tile(&mut self, image_sheet_asset: &ImageSheetAsset, tile: usize, position: [i32; 2], assets: &mut Assets) {
        let img = assets.image_sheet(image_sheet_asset);
        if let Some(texture) = img.get(tile) {
            let transform = self.context.transform.trans(position[0] as f64, position[1] as f64);
            image(texture, transform, self.gl);
        }
    }

    #[deprecated]
    pub(crate) fn texture(&mut self, texture: Texture, position: [f64; 2]) {
        let transform = self.context.transform.trans(position[0], position[1]);
        // Workaround for a behaviour of piston where it passes a reference of the texture to the backend for async rendering,
        // but that texture might be dropped before the frame is rendered, resulting in a black square.
        // So I save the texture in this array, which is dropped after the frame is rendered.
        self.textures.push(texture);
        image(self.textures.last().unwrap(), transform, self.gl);
    }

    #[deprecated]
    pub(crate) fn texture_ref(&mut self, texture: &Texture, position: [f64; 2]) {
        let transform = self.context.transform.trans(position[0], position[1]);
        image(texture, transform, self.gl);
    }

}