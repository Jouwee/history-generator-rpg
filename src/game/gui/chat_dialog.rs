use std::ops::ControlFlow;

use crate::{engine::{gui::{button::Button, containers::SimpleContainer, label::Label, layout_component::LayoutComponent, UIEvent, UINode}, scene::ShowChatDialogData}, game::chunk::TileMetadata, globals::perf::perf, world::{event::Event, world::World, writer::Writer}, GameContext, RenderContext};

pub(crate) struct ChatDialog {
    layout: LayoutComponent,
    inspected: ShowChatDialogData,
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
            inspected,
            chat_container,
            response_container,
        }
    }

    fn who(&mut self, world: &World, game_ctx: &mut GameContext)  {
        let mut writer = Writer::new(&world, &game_ctx.resources);

        writer.add_text("\"Who are you?\", you ask.");

        writer.chat_present_self(&self.inspected.actor);

        let text = &writer.take_text();
        for line in text.split("\n") {
            let line = Label::text(&line);
            self.chat_container.add(line);
        }
    }

    fn quest(&mut self, world: &World, game_ctx: &mut GameContext)  {
        
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

        writer.describe_actor(&self.inspected.actor);

        writer.quote_actor("Yes?", &self.inspected.actor);

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