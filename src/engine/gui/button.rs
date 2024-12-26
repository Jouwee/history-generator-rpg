use std::fmt::Display;

use graphics::{image, CharacterCache, Transformed};
use ::image::ImageReader;
use piston::Button as Btn;

use crate::{engine::{render::RenderContext, spritesheet::Spritesheet, Color}, game::InputEvent};

use super::{GUINode, Position};

pub struct Button {
    text: String,
    position: Position,
    last_layout: [f64; 4]
}

impl Button {
    pub fn new(text: impl Display, position: Position) -> Button {
        Button { text: text.to_string(), position, last_layout: [0.; 4] }
    }

    pub fn text(&mut self, text: impl Display) {
        self.text = text.to_string();
    }

    pub fn event(&self, evt: &InputEvent) -> ButtonEvent {
        if let Btn::Mouse(_) = evt.button_args.button {
            let position = self.last_layout;
            if evt.mouse_pos[0] >= position[0] && evt.mouse_pos[1] >= position[1] && evt.mouse_pos[0] <= position[0]+position[2] && evt.mouse_pos[1] <= position[1]+position[3] {
                return ButtonEvent::Click
            }
        }
        return ButtonEvent::None
    }

}

impl GUINode for Button {
    fn render(&mut self, ctx: &mut RenderContext) {
        let original = ctx.context.transform;
        // Renders on the original transform for pixelated font. Won't work with scaled stuff.
        ctx.context.transform = ctx.original_transform;
        let size = self.min_size(ctx);
        let mut position = self.compute_position(&self.position, self.parent_rect(ctx), size);
        self.last_layout = [position[0], position[1], size[0], size[1]];
        // TODO: Better spritesheets, and scaling
        let spritesheet = ImageReader::open("./assets/sprites/button.png").unwrap().decode().unwrap();
        let spritesheet = Spritesheet::new(spritesheet, (8, 8));
        // Corners
        let transform = ctx.context.transform.trans(position[0], position[1]).scale(2., 2.);
        image(spritesheet.sprite(0, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 16.).scale(2., 2.);
        image(spritesheet.sprite(0, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 16., position[1]).scale(2., 2.);
        image(spritesheet.sprite(2, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 16., position[1] + size[1] - 16.).scale(2., 2.);
        image(spritesheet.sprite(2, 2), transform, ctx.gl);
        // Borders
        let transform = ctx.context.transform.trans(position[0] + 16., position[1]).scale((size[0]-32.) / 8., 2.);
        image(spritesheet.sprite(1, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + 16., position[1] + size[1] - 16.).scale((size[0]-32.) / 8., 2.);
        image(spritesheet.sprite(1, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + 16.).scale(2., (size[1]-32.) / 8.);
        image(spritesheet.sprite(0, 1), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 16., position[1] + 16.).scale(2., (size[1]-32.) / 8.);
        image(spritesheet.sprite(2, 1), transform, ctx.gl);
        // Body
        let transform = ctx.context.transform.trans(position[0] + 16., position[1] + 16.).scale((size[0]-32.) / 8., (size[1]-32.) / 8.);
        image(spritesheet.sprite(1, 1), transform, ctx.gl);
        // Somewhat center text
        position[0] += 4.;
        position[1] += 17.;
        ctx.text(&self.text, 11, position, Color::from_hex("ffffff"));
        ctx.context.transform = original;
    }

    fn min_size(&self, ctx: &mut RenderContext) -> [f64; 2] {
        let width = ctx.default_font.width(12, &self.text);
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