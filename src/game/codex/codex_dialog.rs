use crate::{commons::history_vec::Id, engine::{gui::{button::{Button, ButtonEvent}, container::Container, dialog::Dialog, label::Label, vlist::VList, Anchor, GUINode, Position}, render::RenderContext}, literature::biography::BiographyWriter, world::world::World};

use super::knowledge_codex::KnowledgeCodex;

pub struct CodexDialog {
    dialog: Option<Dialog>,
}

impl CodexDialog {
    pub fn new() -> CodexDialog {
        CodexDialog {
            dialog: None,
        }
    }

    pub fn start_dialog(&mut self) {
        let mut dialog = Dialog::new();
        dialog.add_key("btn_creatures", Button::new("People & Creatures", Position::Anchored(Anchor::TopLeft, 10., 10.)));
        dialog.add_key("btn_places", Button::new("Places", Position::Anchored(Anchor::TopLeft, 160., 10.)));
        dialog.add_key("btn_artifacts", Button::new("Artifacts", Position::Anchored(Anchor::TopLeft, 260., 10.)));
        dialog.add_key("btn_close", Button::new("Close", Position::Anchored(Anchor::BottomRight, 128., 34.)));
        dialog.add_key("selected_info", VList::new(Position::Anchored(Anchor::TopLeft, 210., 52.)));
        self.dialog = Some(dialog);
    }

    pub fn input_state(&mut self, evt: &crate::game::InputEvent, world: &World, codex: &KnowledgeCodex) {
        if let Some(dialog) = &mut self.dialog {
            if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_close").unwrap().event(evt) {
                self.dialog = None;
                return
            }
            if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_creatures").unwrap().event(evt) {
                Self::click_creatures(dialog, world, codex);
                return
            }

            for (id, _knowledge) in codex.known_creatures() {
                if let ButtonEvent::Click = dialog.get_mut::<Button>(format!("creature:{}", id.0).as_str()).unwrap().event(evt) {
                    Self::click_creature(dialog.get_mut::<VList>("selected_info").unwrap(), *id, world, codex);
                }
            }

        }
    }

    fn click_creatures(dialog: &mut Dialog, world: &World, codex: &KnowledgeCodex) {
        let mut y = 44.;
        let writer = BiographyWriter::new(world);
        for (id, _knowledge) in codex.known_creatures() {
            let person = world.people.get(id).unwrap();
            dialog.add_key(format!("creature:{}", id.0).as_str(), Button::new(writer.name(&person), Position::Anchored(Anchor::TopLeft, 10., y)));
            y += 26.;
        }
    }

    fn click_creature(container: &mut VList, id: Id, world: &World, codex: &KnowledgeCodex) {
        let creature = world.people.get(&id).unwrap();
        let knowledge = codex.creature(&id).unwrap();
        let writer = BiographyWriter::new(world);
        container.clear();
        container.add(Label::new(writer.name_with_title(&creature), Position::Auto));
        for event in knowledge.events.iter() {
            let event = world.events.get(*event).unwrap();
            container.add(Label::new(writer.event(&event), Position::Auto));
        }
    }

}

impl GUINode for CodexDialog {
    
    fn render(&mut self, ctx: &mut RenderContext) {
        if let Some(interact_dialog) = &mut self.dialog {
            interact_dialog.render(ctx);
        }
    }

}