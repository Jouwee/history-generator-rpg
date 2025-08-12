use std::ops::ControlFlow;

use crate::{engine::{gui::{button::Button, containers::SimpleContainer, label::Label, layout_component::LayoutComponent, UIEvent, UINode}, scene::BusEvent}, game::{codex::{Quest, QuestStatus}, factory::item_factory::ItemFactory}, globals::perf::perf, world::{item::Item, world::World}, GameContext, RenderContext};

pub(crate) struct QuestCompleteDialog {
    layout: LayoutComponent,
    quest: Quest,
    chat_container: SimpleContainer,
    response_container: SimpleContainer,
    actions_container: SimpleContainer,
    selected_i: usize,
    rewards: Vec<(Button, Item)>,
}

impl QuestCompleteDialog {
    
    pub(crate) fn new(quest: Quest) -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([316., 32.*3.]).padding([8.; 4]);

        let mut chat_container = SimpleContainer::new();
        chat_container.layout_component().size([296., 16.]).anchor_top_center(0., 0.);

        let mut response_container = SimpleContainer::new();
        response_container.layout_component().size([24.*3.+16., 32.]).anchor_top_center(0., 24.);

        let mut actions_container = SimpleContainer::new();
        actions_container.layout_component().size([24.*3.+16., 24.]).anchor_top_center(0., 16.+32.+4.);

        Self {
            layout,
            quest,
            chat_container,
            response_container,
            actions_container,
            rewards: Vec::new(),
            selected_i: 1,
        }
    }

}

impl UINode for QuestCompleteDialog {
    type State = World;
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, _world: &Self::State, game_ctx: &mut GameContext) {

        let label = Label::text("\"I see you have completed my quest! Here, choose a reward.\"");
        self.chat_container.add(label);

        let rewards = ItemFactory::quest_rewards(&game_ctx.resources, 3);
        let mut i = 0;
        for reward in rewards {
            let mut button = Button::text("").tooltip(reward.make_tooltip(&game_ctx.resources.materials));
            button.set_selected(i == self.selected_i);
            self.rewards.push((button, reward));
            i = i + 1;
        }

        let mut button = Button::text("Confirm reward").key("confirm");
        button.layout_component().size([24.*3.+16., 24.]);
        self.actions_container.add(button);
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("inspect");
        self.layout.on_layout(|ctx| {
            self.chat_container.render(&(), ctx, game_ctx);
            self.response_container.layout_component().on_layout(|ctx| {
                let mut x = 0.;
                let ctx_copy = ctx.layout_rect;
                for (button, item) in self.rewards.iter_mut() {
                    ctx.layout_rect[0] = ctx_copy[0] + x;
                    button.render(&(), ctx, game_ctx);
                    ctx.texture_old(item.make_texture(&game_ctx.resources.materials), [ctx.layout_rect[0], ctx.layout_rect[1]]);
                    x += 32.;
                }
            }, ctx);
            self.actions_container.render(&(), ctx, game_ctx);
        }, ctx);
        perf().end("inspect");
    }

    fn input(&mut self, world: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        match self.actions_container.input(&mut (), evt, ctx) {
            ControlFlow::Break(UIEvent::ButtonClicked(button)) => {
                match button.as_str() {
                    "confirm" => {
                        let reward = self.rewards.remove(self.selected_i).1;
                        ctx.event_bus.push(BusEvent::AddItemToPlayer(reward));
                        
                        for quest in world.codex.quests_mut() {
                            if quest.quest_giver == self.quest.quest_giver {
                                quest.status = QuestStatus::Complete;
                            }
                        }

                        return ControlFlow::Break(UIEvent::DialogClosed);
                    }
                    _ => ()
                }
                return ControlFlow::Break(UIEvent::ButtonClicked(button));
            },
            _ => ()
        };
        let mut click = None;
        for (i, (button, _)) in self.rewards.iter_mut().enumerate() {
            if button.input(&mut (), evt, ctx).is_break() {
                click = Some(i);
            }
        }
        if let Some(i) = click {
            self.selected_i = i;
            for (j, (button, _)) in self.rewards.iter_mut().enumerate() {
                button.set_selected(i == j);
            }
            return ControlFlow::Break(UIEvent::ButtonClicked(String::new()));
        }
        return ControlFlow::Continue(())
    }

}