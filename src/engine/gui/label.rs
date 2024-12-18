use std::fmt::Display;

use graphics::CharacterCache;

use crate::engine::{render::RenderContext, Color};

use super::{GUINode, Position};

pub struct Label {
    text: String,
    position: Position,
}

impl Label {
    pub fn new(text: impl Display, position: Position) -> Label {
        Label { text: text.to_string(), position }
    }

    pub fn text(&mut self, text: impl Display) {
        self.text = text.to_string();
    }
}

impl GUINode for Label {
    fn render(&mut self, ctx: &mut RenderContext) {
        let transform = ctx.context.transform;
        // Renders on the original transform for pixelated font. Won't work with scaled stuff.
        ctx.context.transform = ctx.original_transform;
        ctx.text(&self.text, 12, self.compute_position(&self.position, self.parent_rect(ctx), [128., 16.]), Color::from_hex("ffffff"));
        ctx.context.transform = transform;
    }

    fn min_size(&self, ctx: &mut RenderContext) -> [f64; 2] {
        let width = ctx.default_font.width(12, &self.text);
        if let Ok(width) = width {
            return [width, 16.]
        } else {
            return [16., 16.]
        }
    }
}