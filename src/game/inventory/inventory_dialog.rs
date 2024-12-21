use crate::{engine::{gui::{button::{Button, ButtonEvent}, container::Container, dialog::Dialog, vlist::VList, Anchor, GUINode, Position}, render::RenderContext}, world::world::World};

use super::inventory::Inventory;

pub struct InventoryDialog {
    dialog: Option<Dialog>
}

impl InventoryDialog {
    pub fn new() -> InventoryDialog {
        InventoryDialog {
            dialog: None
        }
    }

    pub fn start_dialog(&mut self, inventory: &Inventory, world: &World) {
        let mut dialog = Dialog::new();
        let mut list = VList::new(Position::Anchored(Anchor::TopLeft, 10., 10.));
        Self::build_inventory(&mut list, inventory, world);
        dialog.add_key("list", list);
        dialog.add_key("btn_close", Button::new("Close", Position::Anchored(Anchor::BottomRight, 128., 34.)));
        self.dialog = Some(dialog);
    }

    pub fn build_inventory(container: &mut VList, inventory: &Inventory, world: &World) {
        container.clear();
        for (i, item, equip) in inventory.iter() {
            let mut name = item.name(world);
            if equip {
                name.push_str(" (e)");
            }
            container.add_key(&i.to_string(), Button::new(name, Position::Auto));
        }
    }

    pub fn input_state(&mut self, evt: &crate::game::InputEvent, inventory: &mut Inventory, world: &World) {
        if let Some(dialog) = &mut self.dialog {
            if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_close").unwrap().event(evt) {
                self.dialog = None;
                return
            }
            for i in 0..inventory.len() {
                if let ButtonEvent::Click = dialog.get_mut::<VList>("list").unwrap().get_mut::<Button>(&i.to_string()).unwrap().event(evt) {
                    inventory.equip(i);
                    Self::build_inventory(dialog.get_mut::<VList>("list").unwrap(), inventory, world);
                }
            }
        }
    }

}

impl GUINode for InventoryDialog {
    
    fn render(&mut self, ctx: &mut RenderContext) {
        if let Some(interact_dialog) = &mut self.dialog {
            interact_dialog.render(ctx);
        }
    }

}