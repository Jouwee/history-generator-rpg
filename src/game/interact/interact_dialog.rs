use crate::{engine::{gui::{button::{Button, ButtonEvent}, container::Container, dialog::Dialog, label::Label, Anchor, GUINode, Position}, render::RenderContext}, game::codex::knowledge_codex::{CreatureFact, KnowledgeCodex}, resources::resources::Resources, world::{creature::{Creature, CreatureId}, history_sim::structs::World}, GameContext};

pub struct InteractDialog {
    interact_dialog: Option<Dialog>,
    dialog_y: f64,
    creature: Option<Creature>
}

impl InteractDialog {
    pub fn new() -> InteractDialog {
        InteractDialog {
            interact_dialog: None,
            dialog_y: 0.,
            creature: None,
        }
    }

    pub fn start_dialog(&mut self, world: &World, creature: CreatureId) {
        let mut dialog = Dialog::new();

        self.creature = Some(world.get_creature(&creature).clone());
        self.dialog_y = 0.;

        dialog.add_key("btn_who", Button::new("Who are you?", Position::Anchored(Anchor::BottomLeft, 10., 34.)));
        dialog.add_key("btn_rumor", Button::new("Heard any rumors?", Position::Anchored(Anchor::BottomLeft, 128., 34.)));
        dialog.add_key("btn_close", Button::new("Close", Position::Anchored(Anchor::BottomRight, 128., 34.)));
        self.interact_dialog = Some(dialog);

        self.add_dialog_line("Hi, how can I help you?");
    }

    pub fn add_dialog_line(&mut self, string: &str) {
        if let Some(dialog) = &mut self.interact_dialog {
            dialog.add(Label::new(string, Position::Anchored(Anchor::TopLeft, 10., self.dialog_y + 24.)));
            self.dialog_y = self.dialog_y + 16.;
        }
    }
    pub fn input_state(&mut self, evt: &crate::game::InputEvent, world: &World, resources: &Resources, codex: &mut KnowledgeCodex) {
        if let Some(dialog) = &mut self.interact_dialog {
            if let Some(creature) = &self.creature {
                if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_close").unwrap().event(evt) {
                    self.interact_dialog = None;
                    return
                }
                if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_rumor").unwrap().event(evt) {
                    // TODO:
                    // let rumor = world.events.find_rumor(&Rng::seeded(creature.id), &world,  crate::WorldEventDate { year: 500 }, creature.position);
                    // if let Some((id, rumor)) = rumor {
                    //     codex.add_event(id, rumor);
                    //     self.add_dialog_line(BiographyWriter::new(world, resources).rumor(rumor).as_str());
                    // } else {
                    //     self.add_dialog_line("Sorry, I haven't heard anything.");
                    // }
                    return
                }
                if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_who").unwrap().event(evt) {
                    // TODO:
                    // codex.add_creature_fact(&creature.id, CreatureFact::Name);
                    // self.add_dialog_line(format!("I am {}", creature.name().unwrap()).as_str());
                }
            }
        }
    }

}

impl GUINode for InteractDialog {
    
    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        if let Some(interact_dialog) = &mut self.interact_dialog {
            interact_dialog.render(ctx, game_ctx);
        }
    }

}