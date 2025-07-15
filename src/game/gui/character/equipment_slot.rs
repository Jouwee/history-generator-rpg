use std::ops::ControlFlow;

use piston::MouseButton;

use crate::{engine::gui::{layout_component::LayoutComponent, UINode}, game::inventory::inventory::Inventory, Color, EquipmentType, InputEvent, Item};


pub(crate) struct EquipmentSlot {
    layout: LayoutComponent,
    slot: EquipmentType,
}

impl EquipmentSlot {
    
    pub(crate) fn new(slot: EquipmentType) -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([24., 24.]).padding([1.; 4]);
        Self {
            layout,
            slot
        }
    }

    fn can_place_drag_item(&self, drag_item: &Option<Item>) -> bool {
        if let Some(item) = drag_item {
            if let Some(equippable) = &item.equippable {
                return equippable.slot == self.slot;
            }
        }
        return false
    }

}

impl UINode for EquipmentSlot {
    type State = Inventory;
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout;
    }

    fn render(&mut self, state: &Self::State, ctx: &mut crate::RenderContext, game_ctx: &mut crate::GameContext) {
        let layout = self.layout.compute_layout_rect(ctx.layout_rect);
        ctx.rectangle_fill(layout, Color::from_hex("090714"));
        let layout = self.layout.compute_inner_layout_rect(ctx.layout_rect);
        ctx.rectangle_fill(layout, Color::from_hex("24232a"));
        if let Some(item) = &state.equipped(&self.slot) {
            let texture = item.make_texture(&game_ctx.resources.materials);
            ctx.texture(texture, [layout[0], layout[1]]);
        }
    }

    fn input(&mut self, state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut crate::GameContext) -> ControlFlow<Self::Input> {
        match evt {
            InputEvent::Click { button: MouseButton::Left, pos } => {
                if self.layout.hitbox(pos) {
                    if ctx.drag_item.is_none() || self.can_place_drag_item(&ctx.drag_item) {
                        let mut drag = ctx.drag_item.take();
                        if state.equipped(&self.slot).is_some() {
                            ctx.drag_item = state.unequip(&self.slot);
                        }
                        if let Some(item) = drag.take() {
                            state.equip(&self.slot, item);
                        }
                }
                    return ControlFlow::Break(());
                }
            },
            _ => (),
        }
        return ControlFlow::Continue(());
    }

}