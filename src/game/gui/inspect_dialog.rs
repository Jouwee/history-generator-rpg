use std::ops::ControlFlow;

use crate::{engine::{gui::{containers::SimpleContainer, label::Label, layout_component::LayoutComponent, UIEvent, UINode}, scene::ShowInspectDialogData}, game::chunk::TileMetadata, globals::perf::perf, world::{event::Event, world::World, writer::Writer}, GameContext, RenderContext};

pub(crate) struct InspectDialog {
    layout: LayoutComponent,
    inspected: ShowInspectDialogData,
    info_container: SimpleContainer
}

impl InspectDialog {
    
    pub(crate) fn new(inspected: ShowInspectDialogData) -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([236., 116.]).padding([8.; 4]);

        let mut info_container = SimpleContainer::new();
        info_container.layout_component().anchor_top_left(0., 0.).size([220., 100.]);

        Self {
            layout,
            inspected,
            info_container,
        }
    }

}

impl UINode for InspectDialog {
    type State = World;
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, world: &Self::State, game_ctx: &mut GameContext) {
        let mut writer = Writer::new(&world, &game_ctx.resources);

        if let Some(actor) = &self.inspected.actor {
            writer.describe_actor(actor);
            writer.describe_actor_health(actor);
        }

        if let Some(item) = &self.inspected.item {
            writer.describe_item(item);
        }

        match &self.inspected.tile_metadata {
            Some(TileMetadata::BurialPlace(creature_id)) => {
                writer.describe_burial_place(creature_id);
            },
            None => (),
        };

        let text = &writer.take_text();
        for line in text.split("\n") {
            let line = Label::text(&line);
            self.info_container.add(line);
        }
        
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("inspect");
        self.layout.on_layout(|ctx| self.info_container.render(&(), ctx, game_ctx), ctx);
        perf().end("inspect");
    }

    fn destroy(&mut self, world: &mut Self::State, _game_ctx: &mut GameContext) {
        if let Some(actor) = &self.inspected.actor {
            if let Some(creature_id) = actor.creature_id {
                let codex = world.codex.creature_mut(&creature_id);
                codex.add_appearance();
            }
        }

        if let Some(item) = &self.inspected.item {
            // TODO:
            // if let Some(creature_id) = actor.creature_id {
            //     let codex = world.codex.creature_mut(&creature_id);
            //     codex.add_appearance();
            // }
        }

        match &self.inspected.tile_metadata {
            Some(TileMetadata::BurialPlace(creature_id)) => {
                let events = world.events.iter().enumerate().filter(|(_, evt)| {
                    match evt {
                        Event::CreatureDeath { date: _, creature_id: _, cause_of_death: _ } | Event::BurriedWithPosessions { date: _, creature_id: _, items_ids: _ } => (),
                        _ => return false,
                    };
                    return evt.relates_to_creature(creature_id)
                }).map(|(i, _)| i);
                let codex = world.codex.creature_mut(&creature_id);
                codex.add_name();
                codex.add_birth();
                codex.add_death();
                codex.add_father();
                codex.add_mother();
                for event in events {
                    codex.add_event(event);
                }
            },
            None => (),
        };
    }

    fn input(&mut self, _state: &mut Self::State, _evt: &crate::InputEvent, _ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        return ControlFlow::Continue(())
    }

}