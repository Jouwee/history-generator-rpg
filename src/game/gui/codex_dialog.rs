use std::ops::ControlFlow;

use crate::{engine::{assets::{assets, Assets}, gui::{button::Button, containers::SimpleContainer, label::Label, layout_component::LayoutComponent, UINode}, COLOR_WHITE}, globals::perf::perf, world::{creature::CreatureId, item::ItemId, world::World, writer::Writer}, GameContext, RenderContext};

pub(crate) struct CodexDialog {
    layout: LayoutComponent,
    creatures_button: Button,
    artifacts_button: Button,
    buttons: Vec<(Selection, Button)>,
    selected: Selection,
    info_container: SimpleContainer
}

impl CodexDialog {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([400., 332.]).padding([8.; 4]);

        let mut creatures_button = Button::text("Creatures");
        creatures_button.layout_component().anchor_top_left(16., 16.);
        creatures_button.set_selected(true);

        let mut artifacts_button = Button::text("Artifacts");
        artifacts_button.layout_component().anchor_top_left(80., 16.);

        let mut info_container = SimpleContainer::new();
        info_container.layout_component().anchor_top_left(130., 16.);

        Self {
            layout,
            creatures_button,
            artifacts_button,
            buttons: Vec::new(),
            selected: Selection::None,
            info_container
        }
    }

    fn build_creatures(&mut self, state: &World, game_ctx: &mut GameContext) {
        let mut y = 46.;
        self.buttons.clear();
        for id in state.codex.creatures() {
            let creature = state.creatures.get(id);
            let mut button = Button::text(&creature.name(id, state, &game_ctx.resources));
            button.layout_component().anchor_top_left(24., y);
            self.buttons.push((Selection::Creature(*id), button));
            y += 24.;
        }
    }

    fn build_artifacts(&mut self, state: &World, game_ctx: &mut GameContext) {
        let mut y = 46.;
        self.buttons.clear();
        for id in state.codex.artifacts() {
            let artifact = state.artifacts.get(id);
            let mut button = Button::text(&artifact.name(&game_ctx.resources.materials));
            button.layout_component().anchor_top_left(24., y);
            self.buttons.push((Selection::Artifact(*id), button));
            y += 24.;
        }
    }

    fn update_info(&mut self, world: &World, ctx: &mut GameContext) {
        self.info_container.clear();
        if let Selection::Creature(creature_id) = &self.selected {
            let codex = world.codex.creature(creature_id).expect("Shouldn't have shown the button");
            let creature = world.creatures.get(creature_id);

            let name = Label::text(&self.creature_name(creature_id, world, ctx)).font(Assets::font_heading_asset());
            self.info_container.add(name);

            let birth = match codex.know_birth() {
                true => format!("* {}-{}-{}", creature.birth.year(), creature.birth.month(), creature.birth.day()),
                false => String::from("* ?-?-?")
            };
            let birth = Label::text(&birth);
            self.info_container.add(birth);

            let death = match codex.know_death() {
                true => {
                    if let Some((date, _cause)) = creature.death {
                        format!("+ {}-{}-{}", date.year(), date.month(), date.day())
                    } else {
                        String::from("Alive")
                    }
                    
                },
                false => String::from("+ ?-?-?")
            };
            let death = Label::text(&death);
            self.info_container.add(death);

            if codex.know_father() {
                let name = creature.name(&creature.father, world, &ctx.resources);

                let father = Label::text(&format!("Father: {}", name));
                self.info_container.add(father);
            };

            if codex.know_mother() {
                let name = creature.name(&creature.mother, world, &ctx.resources);

                let mother = Label::text(&format!("Mother: {}", name));
                self.info_container.add(mother);
            };
            
            if codex.events().len() > 0 {
                let event = Label::text(&"Events").font(Assets::font_heading_asset());
                self.info_container.add(event);
            }

            for event_i in codex.events() {
                let event = world.events.get(*event_i).expect("Should not return invalid");

                let event = Label::text(&event.event_text(&ctx.resources, &world));
                self.info_container.add(event);

            }
        }

        if let Selection::Artifact(artifact_id) = &self.selected {
            let codex = world.codex.artifact(artifact_id).expect("Shouldn't have shown the button");
            let artifact = world.artifacts.get(artifact_id);


            let name = Label::text(&artifact.name(&ctx.resources.materials)).font(Assets::font_heading_asset());
            self.info_container.add(name);

            let mut writer = Writer::new(world, &ctx.resources);
            writer.describe_item(&artifact);
            let description = Label::text(&writer.take_text()).font(Assets::font_heading_asset());
            self.info_container.add(description);

            for event_i in codex.events() {
                let event = world.events.get(*event_i).expect("Should not return invalid");

                let event = Label::text(&event.event_text(&ctx.resources, &world));
                self.info_container.add(event);

            }

            // TODO(hu2htwck): Other info

        }


    }

    fn creature_name(&self, creature_id: &CreatureId, world: &World, ctx: &GameContext) -> String {
        let codex = world.codex.creature(creature_id).expect("Shouldn't have shown the button");
        let creature = world.creatures.get(creature_id);

        if codex.know_name() {
            return creature.name(creature_id, world, &ctx.resources);
        }

        return String::from("???????");
    }

}

impl UINode for CodexDialog {
    type State = World;
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, state: &Self::State, game_ctx: &mut GameContext) {
        self.build_creatures(state, game_ctx);
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("codex");
        let copy = ctx.layout_rect;
        ctx.layout_rect = self.layout.compute_inner_layout_rect(ctx.layout_rect);

        self.creatures_button.render(&(), ctx, game_ctx);
        self.artifacts_button.render(&(), ctx, game_ctx);

        for (_id, button) in self.buttons.iter_mut() {
            button.render(&(), ctx, game_ctx);
        }

        self.info_container.render(&(), ctx, game_ctx);

        ctx.layout_rect = copy;

        perf().end("codex");
    }

    fn input(&mut self, state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        if self.creatures_button.input(&mut (), evt, ctx).is_break() {
            self.build_creatures(state, ctx);
            self.artifacts_button.set_selected(false);
            self.creatures_button.set_selected(true);
            return ControlFlow::Break(())
        }
        if self.artifacts_button.input(&mut (), evt, ctx).is_break() {
            self.build_artifacts(state, ctx);
            self.creatures_button.set_selected(false);
            self.artifacts_button.set_selected(true);
            return ControlFlow::Break(())

        }

        for (selection, button) in self.buttons.iter_mut() {
            if button.input(&mut (), evt, ctx).is_break() {
                self.selected = *selection;
                self.update_info(&state, ctx);
                return ControlFlow::Break(())
            }
        }
        return ControlFlow::Continue(())
    }

}

#[derive(Clone, Copy)]
enum Selection {
    None,
    Creature(CreatureId),
    Artifact(ItemId)
}