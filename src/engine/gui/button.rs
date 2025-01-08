use std::fmt::Display;

use graphics::{image, CharacterCache, Transformed};
use ::image::ImageReader;
use opengl_graphics::Texture;
use piston::Button as Btn;

use crate::{engine::{render::RenderContext, spritesheet::Spritesheet, Color}, game::InputEvent};

use super::{GUINode, Position};

pub struct Button {
    text: String,
    icon: Option<Texture>,
    position: Position,
    last_layout: [f64; 4]
}

impl Button {
    pub fn new(text: impl Display, position: Position) -> Button {
        Button {
            text: text.to_string(),
            position,
            last_layout: [0.; 4],
            icon: None
        }
    }

    pub fn new_icon(icon: Texture, position: Position) -> Button {
        Button {
            text: String::new(),
            position,
            last_layout: [0.; 4],
            icon: Some(icon)
        }
    }

    pub fn text(&mut self, text: impl Display) {
        self.text = text.to_string();
    }

    pub fn event(&self, evt: &InputEvent) -> ButtonEvent {
        if let Btn::Mouse(_) = evt.button_args.button {
            let position = self.last_layout;
            if evt.mouse_pos_gui[0] >= position[0] && evt.mouse_pos_gui[1] >= position[1] && evt.mouse_pos_gui[0] <= position[0]+position[2] && evt.mouse_pos_gui[1] <= position[1]+position[3] {
                return ButtonEvent::Click
            }
        }
        return ButtonEvent::None
    }

}

impl GUINode for Button {
    fn render(&mut self, ctx: &mut RenderContext) {
        let size = self.min_size(ctx);
        let mut position = self.compute_position(&self.position, self.parent_rect(ctx), size);
        self.last_layout = [position[0], position[1], size[0], size[1]];
        let spritesheet = ImageReader::open("./assets/sprites/gui/button.png").unwrap().decode().unwrap();
        let spritesheet = Spritesheet::new(spritesheet, (8, 8));
        // Corners
        let transform = ctx.context.transform.trans(position[0], position[1]);
        image(spritesheet.sprite(0, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 8.);
        image(spritesheet.sprite(0, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1]);
        image(spritesheet.sprite(2, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + size[1] - 8.);
        image(spritesheet.sprite(2, 2), transform, ctx.gl);
        // Borders
        let transform = ctx.context.transform.trans(position[0] + 8., position[1]).scale((size[0]-16.) / 8., 1.);
        image(spritesheet.sprite(1, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + 8., position[1] + size[1] - 8.).scale((size[0]-16.) / 8., 1.);
        image(spritesheet.sprite(1, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(spritesheet.sprite(0, 1), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(spritesheet.sprite(2, 1), transform, ctx.gl);
        // Body
        let transform = ctx.context.transform.trans(position[0] + 8., position[1] + 8.).scale((size[0]-16.) / 8., (size[1]-16.) / 8.);
        image(spritesheet.sprite(1, 1), transform, ctx.gl);
        if let Some(icon) = &self.icon {
            let transform = ctx.context.transform.trans(position[0], position[1]);
            image(icon, transform, ctx.gl);
        }
        // Somewhat center text
        position[0] += 4.;
        position[1] += 17.;
        ctx.text_small(&self.text, 5, position, Color::from_hex("ffffff"));
    }

    fn min_size(&self, ctx: &mut RenderContext) -> [f64; 2] {
        if let Some(_) = &self.icon {
            return [24., 24.]
        }
        let width = ctx.small_font.width(5, &self.text);
        if let Ok(width) = width {
            return [width + 8., 24.]
        } else {
            return [24., 24.]
        }
    }

}

pub enum ButtonEvent {
    None,
    Click
}