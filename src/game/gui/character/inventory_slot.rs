use std::ops::ControlFlow;

use piston::MouseButton;

use crate::{engine::gui::{layout_component::LayoutComponent, UIEvent, UINode}, Color, InputEvent, Item};


pub(crate) struct InventorySlot {
    layout: LayoutComponent,
}

impl InventorySlot {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([24., 24.]).padding([1.; 4]);
        Self {
            layout,
        }
    }

}

impl UINode for InventorySlot {
    type State = Option<Item>;
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout;
    }

    fn render(&mut self, state: &Self::State, ctx: &mut crate::RenderContext, game_ctx: &mut crate::GameContext) {
        let layout = self.layout.compute_layout_rect(ctx.layout_rect);
        ctx.rectangle_fill(layout, Color::from_hex("090714"));
        let layout = self.layout.compute_inner_layout_rect(ctx.layout_rect);
        ctx.rectangle_fill(layout, Color::from_hex("24232a"));
        if let Some(item) = &state {
            let texture = item.make_texture(&game_ctx.resources.materials);
            ctx.texture_old(texture, [layout[0], layout[1]]);
        }
    }

    fn input(&mut self, state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut crate::GameContext) -> ControlFlow<Self::Input> {
        match evt {
            InputEvent::Click { button: MouseButton::Left, pos } => {
                if self.layout.hitbox(pos) {
                    let mut drag = ctx.drag_item.take();
                    if state.is_some() {
                        ctx.drag_item = state.take();
                    }
                    if let Some(item) = drag.take() {
                        state.replace(item);
                    }
                    return ControlFlow::Break(UIEvent::None);
                }
            },
            InputEvent::MouseMove { pos } => {
                if self.layout.hitbox(pos) {
                    if let Some(item) = state {
                        ctx.tooltips.show_delayed(&item.make_tooltip(&ctx.resources.materials), *pos);
                    } else {
                        ctx.tooltips.hide();
                    }
                }
            }
            _ => (),
        }
        return ControlFlow::Continue(());
    }

}