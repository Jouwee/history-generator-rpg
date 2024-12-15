use std::fmt::Display;

use piston::Button as Btn;

use crate::{engine::{render::RenderContext, Color}, game::InputEvent};

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
        let transform = ctx.context.transform;
        // Renders on the original transform for pixelated font. Won't work with scaled stuff.
        ctx.context.transform = ctx.original_transform;
        let mut position = self.compute_position(&self.position, self.parent_rect(ctx), [24., 24.]);
        self.last_layout = [position[0], position[1], 24., 24.];
        ctx.image("button.png", position);
        position[0] += 3.; // Somewhat center
        position[1] += 16.;
        ctx.text(&self.text, 12, position, Color::from_hex("ffffff"));
        ctx.context.transform = transform;
    }
}

pub enum ButtonEvent {
    None,
    Click
}