use std::ops::ControlFlow;

use crate::{engine::{assets::Assets, gui::{button::Button, label::Label, layout_component::LayoutComponent, UIEvent, UINode}, scene::BusEvent}, globals::perf::perf, GameContext, RenderContext};

pub(crate) struct DeathDialog {
    layout: LayoutComponent,
    died_label: Label,
    quit_button: Button,
    continue_button: Button,
}

impl DeathDialog {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([248., 84.]).padding([8.; 4]);

        let mut died_label = Label::text("You have died.").font(Assets::font_heading_asset());
        died_label.layout_component().anchor_top_center(0., 0.).size([200., 20.]);

        let mut quit_button = Button::text("Quit to main menu");
        quit_button.layout_component().anchor_top_center(0., 24.).size([200., 20.]);

        let mut continue_button = Button::text("Wait 50 years and play as new character");
        continue_button.layout_component().anchor_top_center(0., 48.).size([200., 20.]);

        Self {
            layout,
            died_label,
            quit_button,
            continue_button,
        }
    }

}

impl UINode for DeathDialog {
    type State = ();
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, _state: &Self::State, _game_ctx: &mut GameContext) {
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("death");
        let copy = ctx.layout_rect;
        ctx.layout_rect = self.layout.compute_inner_layout_rect(ctx.layout_rect);

        self.died_label.render(&(), ctx, game_ctx);
        self.quit_button.render(&(), ctx, game_ctx);
        self.continue_button.render(&(), ctx, game_ctx);

        ctx.layout_rect = copy;

        perf().end("death");
    }

    fn input(&mut self, _state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        if self.quit_button.input(&mut (), evt, ctx).is_break() {
            ctx.event_bus.push(BusEvent::QuitToMenu);
            return ControlFlow::Break(UIEvent::None)
        }
        if self.continue_button.input(&mut (), evt, ctx).is_break() {
            ctx.event_bus.push(BusEvent::CreateNewCharacter);
            return ControlFlow::Break(UIEvent::None)

        }
        return ControlFlow::Continue(())
    }

}