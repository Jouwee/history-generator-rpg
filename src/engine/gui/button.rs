use std::fmt::Display;

use graphics::{image, Transformed};
use ::image::ImageReader;
use opengl_graphics::Texture;
use piston::Button as Btn;

use crate::{engine::{render::RenderContext, scene::Update, sprite::Sprite, spritesheet::Spritesheet, Color}, game::InputEvent, GameContext};

use super::{GUINode, Position};

pub(crate) struct Button {
    text: String,
    background: Texture,
    frame: Spritesheet,
    icon: Option<Texture>,
    position: Position,
    last_layout: [f64; 4],
}

impl Button {
    pub(crate) fn new(text: impl Display, position: Position) -> Button {
        let spritesheet = ImageReader::open("./assets/sprites/gui/button/frame.png").unwrap().decode().unwrap();
        let spritesheet = Spritesheet::new(spritesheet, (8, 8));
        Button {
            text: text.to_string(),
            background: Sprite::new("gui/button/background.png").texture,
            frame: spritesheet,
            position,
            last_layout: [0.; 4],
            icon: None,
        }
    }

    pub(crate) fn event(&self, evt: &InputEvent) -> ButtonEvent {
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

    fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {
    }

    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let size = self.min_size(ctx);
        let mut position = self.compute_position(&self.position, self.parent_rect(ctx), size);
        self.last_layout = [position[0], position[1], size[0], size[1]];
        // Background
        let transform = ctx.context.transform.trans(position[0], position[1]).scale(size[0] / 24., size[1] / 24.);
        image(&self.background, transform, ctx.gl);
        // Corners
        let transform = ctx.context.transform.trans(position[0], position[1]);
        image(self.frame.sprite(0, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 8.);
        image(self.frame.sprite(0, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1]);
        image(self.frame.sprite(2, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + size[1] - 8.);
        image(self.frame.sprite(2, 2), transform, ctx.gl);
        // Borders
        let transform = ctx.context.transform.trans(position[0] + 8., position[1]).scale((size[0]-16.) / 8., 1.);
        image(self.frame.sprite(1, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + 8., position[1] + size[1] - 8.).scale((size[0]-16.) / 8., 1.);
        image(self.frame.sprite(1, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(self.frame.sprite(0, 1), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(self.frame.sprite(2, 1), transform, ctx.gl);
        // Icon
        if let Some(icon) = &self.icon {
            let transform = ctx.context.transform.trans(position[0], position[1]);
            image(icon, transform, ctx.gl);
        }
        if self.text.len() > 0 {
            // Somewhat center text
            position[0] += 4.;
            position[1] += 17.;
            ctx.text(&self.text, game_ctx.assets.font_standard(), [position[0] as i32, position[1] as i32], &Color::from_hex("ffffff"));
        }
    }

    fn min_size(&self, _ctx: &mut RenderContext) -> [f64; 2] {
        if let Some(_) = &self.icon {
            return [24., 24.]
        }
        if self.text.len() == 0 {
            return [24., 24.]
        }
        // TODO: Use asset, but this will probably be removed
        let width = 6 * self.text.len();
        if width > 24 {
            return [width as f64 + 8., 24.]
        } else {
            return [24., 24.]
        }
    }

}

pub(crate) enum ButtonEvent {
    None,
    Click
}