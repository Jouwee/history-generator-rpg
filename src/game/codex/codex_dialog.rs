use crate::{commons::{history_vec::Id, id_vec::Id as VId}, engine::{gui::{button::{Button, ButtonEvent}, container::Container, dialog::Dialog, label::Label, vlist::VList, Anchor, GUINode, Position}, render::RenderContext}, literature::biography::BiographyWriter, world::world::{ArtifactId, World}};

use super::knowledge_codex::{ArtifactFact, KnowledgeCodex};

pub struct CodexDialog {
    dialog: Option<Dialog>,
    view: View
}

impl CodexDialog {
    pub fn new() -> CodexDialog {
        CodexDialog {
            dialog: None,
            view: View::Creatures,
        }
    }

    pub fn start_dialog(&mut self) {
        let mut dialog = Dialog::new();
        dialog.add_key("btn_creatures", Button::new("People & Creatures", Position::Anchored(Anchor::TopLeft, 10., 10.)));
        dialog.add_key("btn_places", Button::new("Places", Position::Anchored(Anchor::TopLeft, 180., 10.)));
        dialog.add_key("btn_artifacts", Button::new("Artifacts", Position::Anchored(Anchor::TopLeft, 260., 10.)));
        dialog.add_key("btn_close", Button::new("Close", Position::Anchored(Anchor::BottomRight, 128., 34.)));
        dialog.add_key("entry_list", VList::new(Position::Anchored(Anchor::TopLeft, 10., 44.)));
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
                self.view = View::Creatures;
                Self::click_creatures(dialog.get_mut::<VList>("entry_list").unwrap(), world, codex);
                return
            }
            if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_places").unwrap().event(evt) {
                self.view = View::Places;
                Self::click_places(dialog.get_mut::<VList>("entry_list").unwrap(), world, codex);
                return
            }
            if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_artifacts").unwrap().event(evt) {
                self.view = View::Artifacts;
                Self::click_artifacts(dialog.get_mut::<VList>("entry_list").unwrap(), world, codex);
                return
            }
            if let View::Creatures = self.view {
                for (id, _knowledge) in codex.known_creatures() {
                    if let ButtonEvent::Click = dialog.get_mut::<VList>("entry_list").unwrap().get_mut::<Button>(format!("creature:{}", id.0).as_str()).unwrap().event(evt) {
                        Self::click_creature(dialog.get_mut::<VList>("selected_info").unwrap(), *id, world, codex);
                    }
                }
            }
            if let View::Places = self.view {
                for (id, _knowledge) in codex.known_places() {
                    if let ButtonEvent::Click = dialog.get_mut::<VList>("entry_list").unwrap().get_mut::<Button>(format!("place:{}", id.0).as_str()).unwrap().event(evt) {
                        Self::click_place(dialog.get_mut::<VList>("selected_info").unwrap(), *id, world, codex);
                    }
                }
            }
            if let View::Artifacts = self.view {
                for (id, _knowledge) in codex.known_artifacts() {
                    if let ButtonEvent::Click = dialog.get_mut::<VList>("entry_list").unwrap().get_mut::<Button>(format!("artifact:{}", id.as_usize()).as_str()).unwrap().event(evt) {
                        Self::click_artifact(dialog.get_mut::<VList>("selected_info").unwrap(), *id, world, codex);
                    }
                }
            }

        }
    }

    fn click_creatures(container: &mut VList, world: &World, codex: &KnowledgeCodex) {
        container.clear();
        let writer = BiographyWriter::new(world);
        for (id, _knowledge) in codex.known_creatures() {
            let person = world.people.get(id).unwrap();
            container.add_key(format!("creature:{}", id.0).as_str(), Button::new(writer.name(&person), Position::Auto));
        }
    }

    fn click_places(container: &mut VList, world: &World, codex: &KnowledgeCodex) {
        container.clear();
        for (id, _knowledge) in codex.known_places() {
            let place = world.settlements.get(id);
            container.add_key(format!("place:{}", id.0).as_str(), Button::new(&place.name, Position::Auto));
        }
    }

    fn click_artifacts(container: &mut VList, world: &World, codex: &KnowledgeCodex) {
        container.clear();
        for (id, _knowledge) in codex.known_artifacts() {
            let item = world.artifacts.get(id);
            container.add_key(format!("artifact:{}", id.as_usize()).as_str(), Button::new(&item.name(&world), Position::Auto));
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

    fn click_place(container: &mut VList, id: Id, world: &World, codex: &KnowledgeCodex) {
        let place = world.settlements.get(&id);
        let knowledge = codex.place(&id).unwrap();
        let writer = BiographyWriter::new(world);
        container.clear();
        container.add(Label::new(&place.name, Position::Auto));
        for event in knowledge.events.iter() {
            let event = world.events.get(*event).unwrap();
            container.add(Label::new(writer.event(&event), Position::Auto));
        }
    }

    fn click_artifact(container: &mut VList, id: ArtifactId, world: &World, codex: &KnowledgeCodex) {
        let artifact = world.artifacts.get(&id);
        let knowledge = codex.artifact(&id).unwrap();
        let writer = BiographyWriter::new(world);
        container.clear();
        container.add(Label::new(artifact.name(&world), Position::Auto));
        if knowledge.facts.contains(&ArtifactFact::Description) {
            container.add(Label::new(artifact.description(&world), Position::Auto));
        }
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

enum View {
    Creatures,
    Places,
    Artifacts,
}