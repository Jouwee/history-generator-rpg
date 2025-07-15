use std::ops::ControlFlow;

use crate::{engine::gui::layout_component::LayoutComponent, GameContext, InputEvent, RenderContext, Update};

pub(crate) mod button;
pub(crate) mod containers;
pub(crate) mod context_menu;
pub(crate) mod dialog;
pub(crate) mod layout_component;
pub(crate) mod label;
pub(crate) mod tooltip;


pub(crate) trait UINode {
    type State;
    type Input;

    fn layout_component(&mut self) -> &mut LayoutComponent;

    fn recompute_layout(&mut self, _game_ctx: &mut GameContext) {}

    fn init(&mut self, _state: &Self::State, _game_ctx: &mut GameContext) {}

    fn destroy(&mut self, _state: &mut Self::State, _game_ctx: &mut GameContext) {}

    fn render(&mut self, _state: &Self::State, _ctx: &mut RenderContext, _game_ctx: &mut GameContext) {}

    fn update(&mut self, _state: &mut Self::State, _update: &Update, _ctx: &mut GameContext) {
    }

    fn input(&mut self, _state: &mut Self::State, _evt: &InputEvent, _ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        return ControlFlow::Continue(());
    }

}