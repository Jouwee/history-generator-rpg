use std::ops::ControlFlow;

use crate::{engine::{gui::{button::Button, containers::SimpleContainer, label::Label, layout_component::LayoutComponent, UIEvent, UINode}, scene::{BusEvent, ShowChatDialogData}}, game::{codex::{Quest, QuestObjective, QuestStatus}, factory::item_factory::ItemFactory}, globals::perf::perf, world::{creature::Profession, unit::{UnitId, UnitType}, world::World, writer::Writer}, GameContext, RenderContext};

pub(crate) struct ChatDialog {
    layout: LayoutComponent,
    data: ShowChatDialogData,
    chat_container: SimpleContainer,
    response_container: SimpleContainer
}

impl ChatDialog {
    
    pub(crate) fn new(inspected: ShowChatDialogData) -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([400., 332.]).padding([8.; 4]);

        let mut chat_container = SimpleContainer::new();
        chat_container.layout_component().anchor_top_left(0., 0.);

        let mut response_container = SimpleContainer::new();
        response_container.layout_component().anchor_bottom_left(0., -56.);

        Self {
            layout,
            data: inspected,
            chat_container,
            response_container,
        }
    }

    fn who(&mut self, world: &mut World, game_ctx: &mut GameContext)  {
        let mut writer = Writer::new(&world, &game_ctx.resources);

        writer.add_text("\"Who are you?\", you ask.");
        writer.chat_present_self(&self.data.actor);

        let text = &writer.take_text();
        for line in text.split("\n") {
            let line = Label::text(&line);
            self.chat_container.add(line);
        }


        if let Some(creature_id) = &self.data.actor.creature_id {
            let creature = world.codex.creature_mut(creature_id);
            creature.add_name();
            creature.add_appearance();
        }
    }

    fn quest(&mut self, world: &mut World, game_ctx: &mut GameContext)  {
        let mut writer = Writer::new(&world, &game_ctx.resources);
        writer.add_text("\"Need help with something?\", you ask.");

        let can_give_quest = match &self.data.actor.creature_id {
            Some(creature_id) => {
                let creature = world.creatures.get(creature_id);
                match creature.profession {
                    Profession::Ruler => true,
                    _ => false,
                }
            },
            None => false,
        };

        if !can_give_quest {
            writer.quote_actor("Not at the moment, no. You should talk to the earl.", &self.data.actor);
            
            let text = &writer.take_text();
            for line in text.split("\n") {
                let line = Label::text(&line);
                self.chat_container.add(line);
            }
            return;
        }

        for unit_id in world.units.iter_ids::<UnitId>() {
            let unit = world.units.get(&unit_id);
            let questable = match unit.unit_type {
                /*UnitType::BanditCamp | */UnitType::VarningrLair => true,
                _ => false,
            };
            if !questable {
                continue;
            }
            if unit.creatures.len() == 0 {
                continue;
            }
            
            if unit.xy.dist_squared(&self.data.world_coord) > 7.*7. {
                continue;
            }

            let quest = Quest::new(self.data.actor.creature_id.unwrap().clone(), QuestObjective::KillCreature(unit.creatures.first().unwrap().clone()));
            writer.chat_explain_quest(&quest, &self.data.actor);
            
            let text = &writer.take_text();
            for line in text.split("\n") {
                let line = Label::text(&line);
                self.chat_container.add(line);
            }

            world.codex.add_quest(quest);
            return;
        }

        writer.quote_actor("These are times of peace, and I have nothing to ask of you. Perphaps ask in other towns?", &self.data.actor);
            
        let text = &writer.take_text();
        for line in text.split("\n") {
            let line = Label::text(&line);
            self.chat_container.add(line);
        }
        return;
    }

}

impl UINode for ChatDialog {
    type State = World;
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, world: &Self::State, game_ctx: &mut GameContext) {
        let mut writer = Writer::new(&world, &game_ctx.resources);

        writer.describe_actor(&self.data.actor);

        let pending_quest = world.codex.quests()
            .filter(|quest| quest.quest_giver == self.data.actor.creature_id.unwrap() && quest.status == QuestStatus::RewardPending)
            .next();

        if let Some(pending_quest) = pending_quest {
            writer.quote_actor("I see you have completed my quest! Here, a reward.", &self.data.actor);

            let reward = ItemFactory::quest_reward(&game_ctx.resources);
            writer.add_text("\nHe gifts you a ");
            writer.describe_item(&reward);

            game_ctx.event_bus.push(BusEvent::AddItemToPlayer(reward));

            // TODO: Mark as complete

            writer.quote_actor("Can I help you with something else?", &self.data.actor);
        } else {
            writer.quote_actor("Yes?", &self.data.actor);    
        }

        let text = &writer.take_text();
        for line in text.split("\n") {
            let line = Label::text(&line);
            self.chat_container.add(line);
        }

        self.response_container.add(Button::text("Who are you?").key("who"));
        self.response_container.add(Button::text("Need help with anything?").key("quest"));
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("inspect");
        self.layout.on_layout(|ctx| {
            self.chat_container.render(&(), ctx, game_ctx);
            self.response_container.render(&(), ctx, game_ctx);
        }, ctx);
        perf().end("inspect");
    }

    fn input(&mut self, world: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        match self.response_container.input(&mut (), evt, ctx) {
            ControlFlow::Break(UIEvent::ButtonClicked(button)) => {
                match button.as_str() {
                    "who" => self.who(world, ctx),
                    "quest" => self.quest(world, ctx),
                    _ => ()
                }
                return ControlFlow::Break(());
            },
            _ => ()
        };
        return ControlFlow::Continue(())
    }

}