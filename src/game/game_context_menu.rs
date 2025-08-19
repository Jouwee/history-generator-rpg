use std::ops::ControlFlow;

use piston::MouseButton;

use crate::{commons::id_vec::Id, engine::gui::{context_menu::{ContextMenu, ContextMenuModel}, layout_component::LayoutComponent, UINode}, game::state::GameState, resources::action::{ActionId, ActionRunner}, warn, Coord2, GameContext, InputEvent};

pub(crate) struct GameContextMenu {
    layout: LayoutComponent,
    menu: Option<ActiveMenuData>,
}

impl GameContextMenu {
    
    pub(crate) fn new() -> Self {
        Self {
            layout: LayoutComponent::new(),
            menu: None,
        }
    }

    pub(crate) fn show(&mut self, actor_index: usize, cursor: Coord2, chunk: &mut GameState, ctx: &mut GameContext, position: [f64; 2]) {
        let actor = chunk.actor(actor_index).unwrap();

        let actions: Vec<(i32, String)> = actor.get_all_available_actions(ctx).iter()
            .map(|id| (*id, ctx.resources.actions.get(id)))
            .filter(|(id, action)| ActionRunner::can_use(id, action, actor_index, cursor, chunk).is_ok())
            .map(|(id, action)| (id.as_usize() as i32, action.name.clone()))
            .collect();

        if actions.len() > 0 {
            let mut menu_data = ActiveMenuData {
                cursor_pos: cursor,
                menu: ContextMenu::new(),
                menu_model: ContextMenuModel {
                    items: actions,
                }
            };
            menu_data.menu.layout_component().anchor_top_left(position[0], position[1]);
            menu_data.menu.init(&menu_data.menu_model, ctx);
            self.menu = Some(menu_data);
        }
    }

    pub(crate) fn close(&mut self) {
        self.menu = None;
    }

}

impl UINode for GameContextMenu {
    type State = ();
    type Input = (Coord2, ActionId);

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn input(&mut self, _state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut crate::GameContext) -> ControlFlow<Self::Input> {
        if let Some(menu) = &mut self.menu {
            if let ControlFlow::Break((idu, _)) = menu.menu.input(&mut menu.menu_model, evt, ctx) {
                let id = ctx.resources.actions.validate_id(idu as usize);
                if let Some(id) = id {
                    let cursor = menu.cursor_pos.clone();
                    self.close();
                    return ControlFlow::Break((cursor, id))
                } else {
                    warn!("No action found for ID {}", idu);
                }
            }
            if let InputEvent::Click { button: MouseButton::Left, pos: _ } = evt {
                self.close();
            }
        }

        ControlFlow::Continue(())
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut crate::RenderContext, game_ctx: &mut crate::GameContext) {
        if let Some(menu) = &mut self.menu {
            menu.menu.render(&menu.menu_model, ctx, game_ctx);
        }
    }

}

struct ActiveMenuData {
    cursor_pos: Coord2,
    menu: ContextMenu,
    menu_model: ContextMenuModel,
}