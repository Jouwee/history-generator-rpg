use std::ops::ControlFlow;

use piston::Key;

use crate::{engine::{assets::{assets, Assets}, geometry::Size2D, gui::{button::Button, containers::SimpleContainer, label::{HorizontalAlignment, Label}, layout_component::LayoutComponent, UIEvent, UINode}, scene::BusEvent, COLOR_BACKDROP}, globals::perf::perf, loc, GameContext, InputEvent, RenderContext};

pub(crate) struct InGameMenu {
    visible: bool,
    layout: LayoutComponent,
    container: SimpleContainer
}

impl InGameMenu {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([180., 20.*3.+8.+32. + 16. + 4.]).padding([8.; 4]);

        let mut container = SimpleContainer::new();
        container.layout_component().anchor_center().size([100., 20.*3.+8. + 16. + 4.]);

        let mut title = Label::text(loc!("ingame-menu-title-paused")).font(Assets::font_heading_asset()).hor_alignment(HorizontalAlignment::Center);
        title.layout_component().size([100., 20.]);
        container.add(title);

        let mut button = Button::text(loc!("ingame-menu-resume")).key("resume");
        button.layout_component().size([100., 20.]);
        container.add(button);

        let mut button = Button::text(loc!("ingame-menu-save-game")).key("save-game");
        button.layout_component().size([100., 20.]);
        container.add(button);

        let mut button = Button::text(loc!("ingame-menu-quit-to-menu")).key("quit");
        button.layout_component().size([100., 20.]);
        container.add(button);

        Self {
            layout,
            visible: false,
            container
        }
    }

    pub(crate) fn is_visible(&self) -> bool {
        return self.visible
    }

    pub(crate) fn show(&mut self) {
        self.visible = true;
    }

    pub(crate) fn hide(&mut self) {
        self.visible = false;
    }

}

impl UINode for InGameMenu {
    type State = ();
    type Input = InGameMenuOption;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        if !self.visible {
            return;
        }
        perf().start("ingame_menu");
        ctx.rectangle_fill(ctx.layout_rect, &COLOR_BACKDROP);
        self.layout.on_layout(|ctx| {

            let sheet = assets().image_sheet("gui/fade_bg.png", Size2D(180, 8));
            sheet.draw_as_scalable(ctx.layout_rect, ctx);

            self.container.render(&mut (), ctx, game_ctx);
        }, ctx);
        perf().end("ingame_menu");
    }

    fn input(&mut self, state: &mut Self::State, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        if !self.visible {
            return ControlFlow::Continue(());
        }
        match self.container.input(state, evt, ctx) {
            ControlFlow::Break(UIEvent::ButtonClicked(btn)) => {
                match btn.as_str() {
                    "resume" => self.hide(),
                    "save-game" => return ControlFlow::Break(InGameMenuOption::SaveGame),
                    "quit" => ctx.event_bus.push(BusEvent::QuitToMenu),
                    _ => ()
                }
            },
            _ => ()
        }
        match evt {
            InputEvent::Key { key: Key::Escape } => self.hide(),
            _ => ()
        }
        ControlFlow::Break(InGameMenuOption::None)
    }

}

pub(crate) enum InGameMenuOption {
    None,
    SaveGame,
}