use piston::MouseButton;

use crate::{engine::gui::{layout_component::LayoutComponent, InputResult, UINode}, Color, InputEvent, Item};


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
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout;
    }

    fn render(&mut self, state: &Self::State, ctx: &mut crate::RenderContext, game_ctx: &mut crate::GameContext) {
        let layout = self.layout.compute_layout_rect(ctx);
        ctx.rectangle_fill(layout, Color::from_hex("090714"));
        let layout = self.layout.compute_inner_layout_rect(ctx);
        ctx.rectangle_fill(layout, Color::from_hex("24232a"));
        if let Some(item) = &state {
            let texture = item.make_texture(&game_ctx.resources.materials);
            ctx.texture(texture, [layout[0], layout[1]]);
        }
    }

    fn input(&mut self, state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut crate::GameContext) -> InputResult<Self::Input> {
        match evt {
            InputEvent::Click { button: MouseButton::Left, pos } => {
                if self.layout.hitbox(pos) {
                    if state.is_some() && ctx.drag_item.is_none() {
                        ctx.drag_item = state.take();
                    } else if state.is_none() && ctx.drag_item.is_some() {
                        state.replace(ctx.drag_item.take().expect("already checked"));
                    }
                    return InputResult::Consume(());
                }
            },
            _ => (),
        }
        return InputResult::None;
    }

}