use std::time::Instant;

use crate::{engine::{assets::{assets, Assets, Font, FontAsset}, gui::{layout_component::LayoutComponent, UIEvent, UINode}, COLOR_WHITE}, GameContext, RenderContext};

/// Stateful label
pub(crate) struct Label {
    layout: LayoutComponent,
    text: String,
    widths: Vec<(usize, f64)>,
    font: FontAsset
}

impl Label {

    pub(crate) fn text(text: &str) -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([24., 24.]);

        let widths = compute_widths(text, assets().font_standard());

        Self {
            layout,
            text: String::from(text),
            widths,
            font: Assets::font_standard_asset()
        }
    }

    pub(crate) fn font(mut self, font: FontAsset) -> Self {
        self.font = font;
        return self;
    }

    fn break_text(&self, max_width: f64) -> Vec<&str> {
        let mut lines = Vec::new();        
        let mut line_width = 0.;
        let mut start = 0;
        for (end, word_width) in self.widths.iter() {
            if line_width + word_width > max_width {
                lines.push(&self.text[start..=*end]);
                start = *end + 1;
                line_width = 0.;
            }
            line_width = line_width + word_width;
        }
        lines.push(&self.text[start..]);

        return lines;
    }

}

impl UINode for Label {
    type State = ();
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn recompute_layout(&mut self, container_layout: [f64; 4], _game_ctx: &mut GameContext) {
        let lines = self.break_text(container_layout[2]);
        let mut assets = assets();
        let font = assets.font(&self.font);
        self.layout.size([container_layout[2], lines.len() as f64 * font.line_height()]);
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, _game_ctx: &mut GameContext) {
        let layout = self.layout.compute_inner_layout_rect(ctx.layout_rect);
        let mut y = layout[1] + 15.;
        let lines = self.break_text(ctx.layout_rect[2]);
        let mut assets = assets();
        let font = assets.font(&self.font);
        let line_height = font.line_height();
        let x = layout[0]as i32 + 4;
        for line in lines {
            ctx.text_shadow(&line, font, [x, y as i32], &COLOR_WHITE);   
            y += line_height;
        }
    }

}

fn compute_widths(text: &str, font: &mut Font) -> Vec<(usize, f64)> {
    let mut widths = Vec::new();
    let mut start = 0;

    for (end, char) in text.chars().enumerate() { 
        if end == (text.len() - 1) || char == ' ' {
            let word_width = font.width(&text[start..=end]);
            widths.push((end, word_width));
            start = end;
        }
    }

    return widths;
}