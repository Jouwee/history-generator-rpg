use std::{ops::ControlFlow, sync::Arc};

use graphics::Transformed;
use crate::{engine::{assets::{assets, Image}, gui::{button::Button, containers::SimpleContainer, UIEvent, UINode}, input::InputEvent, render::RenderContext, scene::{Scene, Update}}, loc, GameContext};

pub(crate) struct MainMenuScene {
    logo: Arc<Image>,
    container: SimpleContainer,
}

impl MainMenuScene {
    pub(crate) fn new() -> Self {
        let mut container = SimpleContainer::new();
        container.layout_component().anchor_center().size([124., 28. * 3. - 4.]);
        
        let mut new_game = Button::text(loc!("main-menu-new-game")).key("new_game");
        new_game.layout_component().size([124., 24.]);
        container.add(new_game);

        let mut load_game = Button::text(loc!("main-menu-continue")).key("load_game");
        load_game.layout_component().size([124., 24.]);
        container.add(load_game);

        let mut quit = Button::text(loc!("main-menu-quit")).key("quit");
        quit.layout_component().size([124., 24.]);
        container.add(quit);

        Self {
            logo: assets().image("logo_small.png"),
            container,
        }
    }

}

impl Scene for MainMenuScene {
    type Input = MainMenuOption;

    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        ctx.scale(2.);
        self.container.render(&(), ctx, game_ctx);

        let w = self.logo.size.x() as f64 / 2.;
        let h = self.logo.size.y() as f64 / 2.;
        let x = ctx.layout_rect[2] / 2. - w / 2.;
        let y = self.container.layout_component().last_layout[1] - h as f64 - 16.;
        ctx.texture(&self.logo.texture, ctx.at(x, y).scale(0.5, 0.5));

    }

    fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {
    }

    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<MainMenuOption> {
        match self.container.input(&mut (), &evt, ctx) {
            ControlFlow::Break(UIEvent::ButtonClicked(button)) => {
                match button.as_str() {
                    "new_game" => ControlFlow::Break(MainMenuOption::NewGame),
                    "load_game" => ControlFlow::Break(MainMenuOption::LoadGame),
                    "quit" => ControlFlow::Break(MainMenuOption::Quit),
                    _ => ControlFlow::Continue(()),
                }
            }
            _ => ControlFlow::Continue(()),
        }
    }

    fn event(&mut self, _evt: &crate::engine::scene::BusEvent, _ctx: &mut GameContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

pub(crate) enum MainMenuOption {
    NewGame,
    LoadGame,
    Quit,
}