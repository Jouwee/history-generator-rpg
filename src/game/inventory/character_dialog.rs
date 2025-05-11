use crate::{engine::{gui::{button::{Button, ButtonEvent}, container::Container, dialog::Dialog, label::Label, vlist::VList, Anchor, GUINode, Position}, render::RenderContext}, game::actor::{actor::Actor, health_component::BodyPart}, globals::perf::perf, resources::resources::Resources, world::attributes::Attributes, GameContext};

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

        let mut stats = VList::new(Position::Anchored(Anchor::TopLeft, 10., 10.));
        Self::build_stats(&mut stats, &actor);
        dialog.add_key("stats", stats);

        let mut inventory = VList::new(Position::Anchored(Anchor::TopLeft, 400., 10.));
        Self::build_inventory(&mut inventory, &actor.inventory, resources);
        dialog.add_key("inventory", inventory);

        let mut attributes = VList::new(Position::Anchored(Anchor::TopLeft, 300., 10.));
        Self::build_attributes(&mut attributes, &actor.attributes);
        dialog.add_key("attributes", attributes);

        dialog.add_key("btn_close", Button::new("Close", Position::Anchored(Anchor::BottomRight, 128., 34.)));

        self.dialog = Some(dialog);
    }

    pub(crate) fn build_stats(container: &mut impl Container, actor: &Actor) {
        container.clear();
        container.add(Label::new(format!("HP: {} / {}", actor.hp.health_points(), actor.hp.max_health_points()), Position::Auto));
        container.add(Label::new(format!("AP: {} / {}", actor.ap.action_points, actor.ap.max_action_points), Position::Auto));
        container.add(Label::new(format!("Stamina: {} / {}", actor.stamina.stamina, actor.stamina.max_stamina), Position::Auto));

        container.add(Label::new(format!("Body parts health"), Position::Auto));
        container.add(Label::new(format!("Head: {}%", actor.hp.body_part_condition(&BodyPart::Head).unwrap().condition() * 100.), Position::Auto));
        container.add(Label::new(format!("Torso: {}%", actor.hp.body_part_condition(&BodyPart::Torso).unwrap().condition() * 100.), Position::Auto));
        container.add(Label::new(format!("Left arm: {}%", actor.hp.body_part_condition(&BodyPart::LeftArm).unwrap().condition() * 100.), Position::Auto));
        container.add(Label::new(format!("Right arm: {}%", actor.hp.body_part_condition(&BodyPart::RightArm).unwrap().condition() * 100.), Position::Auto));
        container.add(Label::new(format!("Left leg: {}%", actor.hp.body_part_condition(&BodyPart::LeftLeg).unwrap().condition() * 100.), Position::Auto));
        container.add(Label::new(format!("Right legt: {}%", actor.hp.body_part_condition(&BodyPart::RightLeg).unwrap().condition() * 100.), Position::Auto));

        let stats = actor.stats();
        container.add(Label::new(format!("Crit chance: {}%", stats.critical_hit_chance() * 100.), Position::Auto));
        container.add(Label::new(format!("Crit damage: {}", stats.critical_hit_multiplier()), Position::Auto));
        container.add(Label::new(format!("Dodge change: {}%", stats.dodge_chance() * 100.), Position::Auto));
        container.add(Label::new(format!("Movement AP mult: {}", stats.walk_ap_multiplier()), Position::Auto));
        

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
        perf().start("character_dialog");
        if let Some(interact_dialog) = &mut self.dialog {
            interact_dialog.render(ctx, game_ctx);
        }
        perf().end("character_dialog");
    }

}