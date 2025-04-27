use std::fmt::Display;

use graphics::CharacterCache;

use crate::{engine::{render::RenderContext, Color}, GameContext};

use super::{GUINode, Position};

pub(crate) struct Label {
    text: String,
    position: Position,
}

impl Label {
    pub(crate) fn new(text: impl Display, position: Position) -> Label {
        Label { text: text.to_string(), position }
    }

}

impl GUINode for Label {
    fn render(&mut self, ctx: &mut RenderContext, _game_ctx: &mut GameContext) {
        let mut position = self.compute_position(&self.position, self.parent_rect(ctx), [128., 16.]);
        // Increments y-position because text is rendered bottom-up, everything else is top-down. This normalizes labels to be top-down
        position[1] += 7.;
        ctx.text_small(&self.text, 5, position, Color::from_hex("ffffff"));
    }

    fn min_size(&self, ctx: &mut RenderContext) -> [f64; 2] {
        let width = ctx.default_font.width(12, &self.text);
        if let Ok(width) = width {
            return [width, 7.]
        } else {
            return [16., 7.]
        }
    }
}