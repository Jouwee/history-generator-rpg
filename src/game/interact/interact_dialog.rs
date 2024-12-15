use crate::{commons::history_vec::Id, engine::{gui::{button::{Button, ButtonEvent}, container::Container, dialog::Dialog, label::Label, Anchor, GUINode, Position}, render::RenderContext}, world::{person::Person, world::World}};

pub struct InteractDialog {
    interact_dialog: Option<Dialog>,
    dialog_y: f64,
    person: Option<Person>
}

impl InteractDialog {
    pub fn new() -> InteractDialog {
        InteractDialog {
            interact_dialog: None,
            dialog_y: 0.,
            person: None
        }
    }

    pub fn start_dialog(&mut self, world: &World, creature: Id) {
        let mut dialog = Dialog::new();

        self.person = Some(world.people.get(&creature).unwrap().clone());
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

    pub fn ask_who(&mut self, person: &Person) {
        self.add_dialog_line("> Who are you?");
        self.add_dialog_line(format!("I am {}", person.name().unwrap()).as_str());
    }

}

impl GUINode for InteractDialog {
    
    fn render(&mut self, ctx: &mut RenderContext) {
        if let Some(interact_dialog) = &mut self.interact_dialog {
            interact_dialog.render(ctx);
        }
    }

    fn input(&mut self, evt: &crate::game::InputEvent) {
        if let Some(dialog) = &mut self.interact_dialog {
            if let Some(person) = &self.person {
                if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_close").unwrap().event(evt) {
                    self.interact_dialog = None;
                    return
                }
                if let ButtonEvent::Click = dialog.get_mut::<Button>("btn_who").unwrap().event(evt) {
                    self.add_dialog_line(format!("I am {}", person.name().unwrap()).as_str());
                }
            }
        }
    }

}