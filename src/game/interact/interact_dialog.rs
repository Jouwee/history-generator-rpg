use crate::{engine::{gui::{button::{Button, ButtonEvent}, container::Container, dialog::Dialog, label::Label, Anchor, GUINode, Position}, render::RenderContext}, world::{creature::{Creature, CreatureId}, world::World}, GameContext};

pub(crate) struct InteractDialog {
    interact_dialog: Option<Dialog>,
    dialog_y: f64,
    creature: Option<(CreatureId, Creature)>
}

impl InteractDialog {
    pub(crate) fn new() -> InteractDialog {
        InteractDialog {
            interact_dialog: None,
            dialog_y: 0.,
            creature: None,
        }
    }

    pub(crate) fn start_dialog(&mut self, world: &World, creature_id: CreatureId) {
        let mut dialog = Dialog::new();

        self.creature = Some((creature_id, world.creatures.get(&creature_id).clone()));
        self.dialog_y = 0.;

        dialog.add_key("btn_who", Button::new("Who are you?", Position::Anchored(Anchor::BottomLeft, 10., 34.)));
        dialog.add_key("btn_rumor", Button::new("Heard any rumors?", Position::Anchored(Anchor::BottomLeft, 128., 34.)));
        dialog.add_key("btn_close", Button::new("Close", Position::Anchored(Anchor::BottomRight, 128., 34.)));
        self.interact_dialog = Some(dialog);

        self.add_dialog_line("Hi, how can I help you?");
    }

    pub(crate) fn add_dialog_line(&mut self, string: &str) {
        if let Some(dialog) = &mut self.interact_dialog {
            dialog.add(Label::new(string, Position::Anchored(Anchor::TopLeft, 10., self.dialog_y + 24.)));
            self.dialog_y = self.dialog_y + 16.;
        }
    }
    
    pub(crate) fn input_state(&mut self, evt: &crate::game::InputEvent) {
        if let Some(dialog) = &mut self.interact_dialog {
            if let Some(creature) = &self.creature {
                if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_close").unwrap().event(evt) {
                    self.interact_dialog = None;
                    return
                }
                if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_who").unwrap().event(evt) {
                    // TODO:
                    self.add_dialog_line(format!("I am {:?}", creature.0).as_str());
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