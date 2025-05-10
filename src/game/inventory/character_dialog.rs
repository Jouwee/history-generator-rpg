use crate::{engine::{gui::{button::{Button, ButtonEvent}, container::Container, dialog::Dialog, label::Label, vlist::VList, Anchor, GUINode, Position}, render::RenderContext}, game::actor::actor::Actor, resources::resources::Resources, world::attributes::Attributes, GameContext};

use super::inventory::Inventory;

pub(crate) struct CharacterDialog {
    dialog: Option<Dialog>
}

impl CharacterDialog {
    pub(crate) fn new() -> CharacterDialog {
        CharacterDialog {
            dialog: None
        }
    }

    pub(crate) fn start_dialog(&mut self, actor: &Actor, resources: &Resources) {
        let mut dialog = Dialog::new();
        let mut inventory = VList::new(Position::Anchored(Anchor::TopLeft, 10., 10.));
        Self::build_inventory(&mut inventory, &actor.inventory, resources);
        dialog.add_key("inventory", inventory);
        let mut attributes = VList::new(Position::Anchored(Anchor::TopLeft, 400., 10.));
        Self::build_attributes(&mut attributes, &actor.attributes);
        dialog.add_key("attributes", attributes);
        dialog.add_key("btn_close", Button::new("Close", Position::Anchored(Anchor::BottomRight, 128., 34.)));
        self.dialog = Some(dialog);
    }

    pub(crate) fn build_attributes(container: &mut impl Container, attributes: &Attributes) {
        container.clear();
        let unspent = attributes.unallocated;
        let has_unspent = unspent > 0;
        container.add(Label::new(format!("Points to spend: {unspent}"), Position::Auto));
        container.add(Label::new(format!("strength: {}", attributes.strength), Position::Auto));
        if has_unspent {
            container.add_key("add_str", Button::new("+1 to strength", Position::Auto));
        }
        container.add(Label::new(format!("Agility: {}", attributes.agility), Position::Auto));
        if has_unspent {
            container.add_key("add_agi", Button::new("+1 to agility", Position::Auto));
        }
        container.add(Label::new(format!("Constitution: {}", attributes.constitution), Position::Auto));
        if has_unspent {
            container.add_key("add_con", Button::new("+1 to constitution", Position::Auto));
        }
    }

    pub(crate) fn build_inventory(container: &mut impl Container, inventory: &Inventory, resources: &Resources) {
        container.clear();
        for (i, item, equip) in inventory.iter() {
            let mut name = item.name(&resources.materials);
            if equip {
                name.push_str(" (e)");
            }
            container.add_key(&i.to_string(), Button::new(name, Position::Auto));
        }
    }

    pub(crate) fn input_state(&mut self, evt: &crate::game::InputEvent, actor: &mut Actor, resources: &Resources) -> CharacterDialogOutput {
        if let Some(dialog) = &mut self.dialog {
            if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_close").unwrap().event(evt) {
                self.dialog = None;
                return CharacterDialogOutput::None;
            }
            for i in 0..actor.inventory.len() {
                if let ButtonEvent::Click = dialog.get_mut::<VList>("inventory").unwrap().get_mut::<Button>(&i.to_string()).unwrap().event(evt) {
                    actor.inventory.equip(i);
                    Self::build_inventory(dialog.get_mut::<VList>("inventory").unwrap(), &actor.inventory, resources);
                    return CharacterDialogOutput::EquipmentChanged;
                }
            }
            if let Some(button) = dialog.get_mut::<VList>("attributes").unwrap().get_mut::<Button>("add_str") {
                if let ButtonEvent::Click = button.event(evt) {
                    actor.attributes.strength = actor.attributes.strength + 1;
                    actor.attributes.unallocated = actor.attributes.unallocated - 1;
                    Self::build_attributes(dialog.get_mut::<VList>("attributes").unwrap(), &actor.attributes);
                    return CharacterDialogOutput::None;
                }
            }
            if let Some(button) = dialog.get_mut::<VList>("attributes").unwrap().get_mut::<Button>("add_agi") {
                if let ButtonEvent::Click = button.event(evt) {
                    actor.attributes.agility = actor.attributes.agility + 1;
                    actor.attributes.unallocated = actor.attributes.unallocated - 1;
                    Self::build_attributes(dialog.get_mut::<VList>("attributes").unwrap(), &actor.attributes);
                    return CharacterDialogOutput::None;
                }
            }
            if let Some(button) = dialog.get_mut::<VList>("attributes").unwrap().get_mut::<Button>("add_con") {
                if let ButtonEvent::Click = button.event(evt) {
                    actor.attributes.constitution = actor.attributes.constitution + 1;
                    actor.attributes.unallocated = actor.attributes.unallocated - 1;
                    Self::build_attributes(dialog.get_mut::<VList>("attributes").unwrap(), &actor.attributes);
                    return CharacterDialogOutput::None;
                }
            }
            
        }
        return CharacterDialogOutput::None;
    }

}

pub(crate) enum CharacterDialogOutput {
    None,
    EquipmentChanged
}

impl GUINode for CharacterDialog {
    
    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        if let Some(interact_dialog) = &mut self.dialog {
            interact_dialog.render(ctx, game_ctx);
        }
    }

}