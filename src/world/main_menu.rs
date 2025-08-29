use std::{ops::ControlFlow, sync::Arc};

use graphics::Transformed;
use crate::{engine::{assets::{assets, Image}, geometry::Size2D, gui::{button::Button, containers::SimpleContainer, label::Label, UIEvent, UINode}, input::InputEvent, render::RenderContext, scene::{Scene, Update}}, loadsave::{SaveFile, SaveMetadata}, loc, loc_date, GameContext};

pub(crate) struct MainMenuScene {
    logo: Arc<Image>,
    container: SimpleContainer,
}

impl MainMenuScene {
    pub(crate) fn new() -> Self {
        let mut container = SimpleContainer::new();
        container.layout_component().anchor_center().size([124., 28. * 3. - 4.]);
        
        let mut menu = Self {
            logo: assets().image("logo_small.png"),
            container,
        };
        menu.build_main_menu();
        return menu
    }


    fn build_main_menu(&mut self) {
        self.container.clear();
        self.container.layout_component().anchor_center().size([124., 28. * 3. - 4.]);

        let mut new_game = Button::text(loc!("main-menu-new-game")).key("new_game");
        new_game.layout_component().size([124., 24.]);
        self.container.add(new_game);

        let mut load_game = Button::text(loc!("main-menu-load-game")).key("load_game");
        load_game.layout_component().size([124., 24.]);
        self.container.add(load_game);

        let mut quit = Button::text(loc!("main-menu-quit")).key("quit");
        quit.layout_component().size([124., 24.]);
        self.container.add(quit);
    }

    fn build_load_menu(&mut self) {
        self.container.clear();
        self.container.layout_component().anchor_center().size([200., 232.]);

        let mut inner = SimpleContainer::new();
        inner.layout_component().anchor_center().size([200., 200.]);

        let save_files = SaveFile::enumerate_saves();
        if let Ok(save_files) = save_files {
            for save_file in save_files {
                inner.add(Self::build_save_panel(&save_file));
            }
        }
        self.container.add(inner);

        let mut quit = Button::text(loc!("main-menu-load-back")).key("back");
        quit.layout_component().size([124., 24.]);
        self.container.add(quit);
    }

    fn build_save_panel(save: &SaveMetadata) -> SimpleContainer {
        let mut container = SimpleContainer::new()
            .layout(|l| { l.size([200., 62.]).padding([8.; 4]); })
            .background(assets().image_sheet("gui/fade_bg.png", Size2D(180, 8)));

        let label = Label::text(&save.save_name.to_string());
        container.add(label);

        let label = Label::text(&format!("Last played: {}", loc_date!(&save.last_played)));
        container.add(label);

        let button = Button::text("Play")
            .key(&format!("play:{}", save.save_file_name))
            .layout(|l| { l.size([32., 16.]); });
        container.add(button);

        return container;
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
        let y = self.container.layout_component().last_layout[1] - h as f64;
        ctx.texture(&self.logo.texture, ctx.at(x, y).scale(0.5, 0.5));
    }

    fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {
    }

    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<MainMenuOption> {
        match self.container.input(&mut (), &evt, ctx) {
            ControlFlow::Break(UIEvent::ButtonClicked(button)) => {
                match button.as_str() {
                    "new_game" => ControlFlow::Break(MainMenuOption::NewGame),
                    "load_game" => {
                        self.build_load_menu();
                        ControlFlow::Continue(())
                    },
                    "back" => {
                        self.build_main_menu();
                        ControlFlow::Continue(())
                    },
                    "quit" => ControlFlow::Break(MainMenuOption::Quit),
                    other => {
                        if other.starts_with("play:") {
                            let save_file = other.split(":").last().unwrap();
                            ControlFlow::Break(MainMenuOption::LoadGame(save_file.to_string()))
                        } else {
                            ControlFlow::Continue(())
                        }
                    }
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
    LoadGame(String),
    Quit,
}