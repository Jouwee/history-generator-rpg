use crate::{engine::{assets::{assets, Assets, FontAsset}, gui::{layout_component::LayoutComponent, UINode}, COLOR_WHITE}, GameContext, RenderContext};


/// Stateful label
pub(crate) struct Label {
    layout: LayoutComponent,
    text: String,
    font: FontAsset
}

impl Label {

    pub(crate) fn text(text: &str) -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([24., 24.]);

        Self {
            layout,
            text: String::from(text),
            font: Assets::font_standard_asset()
        }
    }

    pub(crate) fn font(mut self, font: FontAsset) -> Self {
        self.font = font;
        return self;
    }

}

impl UINode for Label {
    type State = ();
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn recompute_layout(&mut self, _game_ctx: &mut GameContext) {
        let mut assets = assets();
        let font = assets.font(&self.font);
        let with = font.width(&self.text);
        // TODO: Line-break
        self.layout.size([with, font.line_height()]);
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, _game_ctx: &mut GameContext) {
        let layout = self.layout.compute_inner_layout_rect(ctx.layout_rect);
        ctx.text_shadow(&self.text, assets().font(&self.font), [layout[0]as i32 + 4, layout[1] as i32 + 15], &COLOR_WHITE);
    }

}