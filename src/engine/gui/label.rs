use std::fmt::Display;

use crate::{engine::{asset::font::FontAsset, render::RenderContext, Color}, GameContext};

use super::{GUINode, Position};

pub(crate) struct Label {
    text: String,
    position: Position,
    font: FontAsset,
}

impl Label {

    pub(crate) fn new(text: impl Display, position: Position) -> Label {
        Label { 
            text: text.to_string(),
            position,
            font: FontAsset::new("Everyday_Standard.ttf", 6)
        }
    }

}

impl GUINode for Label {
    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let mut position = self.compute_position(&self.position, self.parent_rect(ctx), [128., 16.]);
        // Increments y-position because text is rendered bottom-up, everything else is top-down. This normalizes labels to be top-down
        position[1] += 7.;
        ctx.text(&self.text, game_ctx.assets.font(&self.font), [position[0] as i32, position[1] as i32], &Color::from_hex("ffffff"));
    }

    fn min_size(&self, _ctx: &mut RenderContext) -> [f64; 2] {
        // TODO: Should come from asset, but this will be removed
        let width = 13 * self.text.len();
        if width > 0 {
            return [width as f64, 7.]
        } else {
            return [16., 7.]
        }
    }
}