use std::ops::ControlFlow;

use graphics::Transformed;

use crate::{engine::{assets::{assets, Assets}, gui::{button::Button, containers::SimpleContainer, label::Label, layout_component::LayoutComponent, UIEvent, UINode}}, game::codex::{Quest, QuestObjective, QuestStatus}, globals::perf::perf, world::{creature::CreatureId, item::ItemId, site::{SiteId, SiteType}, world::World, writer::Writer}, GameContext, RenderContext};

pub(crate) struct CodexDialog {
    layout: LayoutComponent,
    creatures_button: Button,
    sites_button: Button,
    artifacts_button: Button,
    quests_button: Button,
    buttons: Vec<(Selection, Button)>,
    selected: Selection,
    info_container: SimpleContainer
}

impl CodexDialog {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([400., 332.]).padding([8.; 4]);

        let mut creatures_button = Button::text("Creatures");
        creatures_button.layout_component().anchor_top_left(0., 0.).size([56., 16.]);
        creatures_button.set_selected(true);

        let mut sites_button = Button::text("Sites");
        sites_button.layout_component().anchor_top_left(58., 0.).size([56., 16.]);

        let mut artifacts_button = Button::text("Artifacts");
        artifacts_button.layout_component().anchor_top_left(0., 18.).size([56., 16.]);

        let mut quests_button = Button::text("Quests");
        quests_button.layout_component().anchor_top_left(58., 18.).size([56., 16.]);

        let mut info_container = SimpleContainer::new();
        info_container.layout_component().anchor_top_left(124., 0.).size([256., 316.]);

        Self {
            layout,
            creatures_button,
            artifacts_button,
            sites_button,
            quests_button,
            buttons: Vec::new(),
            selected: Selection::None,
            info_container
        }
    }

    fn build_creatures(&mut self, state: &World, game_ctx: &mut GameContext) {
        let mut y = 40.;
        self.buttons.clear();
        for id in state.codex.creatures() {
            let mut button = Button::text(&&self.creature_name(id, state, &game_ctx));
            button.layout_component().anchor_top_left(0., y).size([114., 16.]);
            self.buttons.push((Selection::Creature(*id), button));
            y += 16.;
        }
    }

    fn build_artifacts(&mut self, state: &World, game_ctx: &mut GameContext) {
        let mut y = 40.;
        self.buttons.clear();
        for id in state.codex.artifacts() {
            let artifact = state.artifacts.get(id);
            let mut button = Button::text(&artifact.name(&game_ctx.resources.materials));
            button.layout_component().anchor_top_left(0., y).size([114., 16.]);
            self.buttons.push((Selection::Artifact(*id), button));
            y += 16.;
        }
    }

    fn build_sites(&mut self, state: &World, _game_ctx: &mut GameContext) {
        let mut y = 40.;
        self.buttons.clear();
        for id in state.codex.sites() {
            let site = state.sites.get(id);
            let mut button = Button::text(site.name());
            button.layout_component().anchor_top_left(0., y).size([114., 16.]);
            self.buttons.push((Selection::Site(*id), button));
            y += 16.;
        }
    }

    fn build_quests(&mut self, state: &World, _game_ctx: &mut GameContext) {
        let mut y = 40.;
        self.buttons.clear();
        for quest in state.codex.quests() {
            let mut name = quest_name(quest);
            if quest.status == QuestStatus::Complete {
                name = name + "[OK]"
            }

            let mut button = Button::text(&name);
            button.layout_component().anchor_top_left(0., y).size([114., 16.]);
            self.buttons.push((Selection::Quest(quest.clone()), button));
            y += 16.;
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
                let father = world.creatures.get(&creature.father);
                let name = father.name(&creature.father, world, &ctx.resources);

                let father = Label::text(&format!("Father: {}", name));
                self.info_container.add(father);
            };

            if codex.know_mother() {
                let mother = world.creatures.get(&creature.mother);
                let name = mother.name(&creature.mother, world, &ctx.resources);

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
            let description = Label::text(&writer.take_text());
            self.info_container.add(description);

            for event_i in codex.events() {
                let event = world.events.get(*event_i).expect("Should not return invalid");

                let event = Label::text(&event.event_text(&ctx.resources, &world));
                self.info_container.add(event);

            }

            // TODO(hu2htwck): Other info

        }

        if let Selection::Quest(quest) = &self.selected {

            
            let mut writer = Writer::new(world, &ctx.resources);
            writer.describe_quest(&quest);

            let name = Label::text(&quest_name(quest)).font(Assets::font_heading_asset());
            self.info_container.add(name);

            let description = Label::text(&writer.take_text());
            self.info_container.add(description);

            let status = match quest.status {
                QuestStatus::Complete => "Complete",
                QuestStatus::InProgress => "In progress",
                QuestStatus::RewardPending => "Retrieve reward",
            };

            let description = Label::text(status);
            self.info_container.add(description);

        }

        if let Selection::Site(site_id) = &self.selected {
            let site = world.sites.get(site_id);

            let name = Label::text(&site.name()).font(Assets::font_heading_asset());
            self.info_container.add(name);

            let text = match &site.site_type {
                SiteType::Village => String::from("A village."),
                SiteType::BanditCamp => String::from("A camp of bandits."),
                SiteType::VarningrLair => String::from("A lair of a Varningr."),
                SiteType::WolfPack => String::from("A den of wolves."),
            };

            let description = Label::text(&text);
            self.info_container.add(description);
        }


    }

    fn creature_name(&self, creature_id: &CreatureId, world: &World, ctx: &GameContext) -> String {
        let codex = world.codex.creature(creature_id).expect("Shouldn't have shown the button");
        let creature = world.creatures.get(creature_id);

        if codex.know_name() {
            let name = creature.name(creature_id, world, &ctx.resources);
            if world.is_played_creature(creature_id) {
                return name + " (You)"
            }
            return name;
        }

        return String::from("???????");
    }

}

impl UINode for CodexDialog {
    type State = World;
    type Input = UIEvent;

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
        self.sites_button.render(&(), ctx, game_ctx);
        self.artifacts_button.render(&(), ctx, game_ctx);
        self.quests_button.render(&(), ctx, game_ctx);

        for (_id, button) in self.buttons.iter_mut() {
            button.render(&(), ctx, game_ctx);
        }

        let transform = ctx.at(ctx.layout_rect[0] + 120., ctx.layout_rect[1]).scale(1., 2.).rot_deg(90.);
        ctx.texture(&assets().image("gui/divider_a.png").texture, transform);

        self.info_container.render(&(), ctx, game_ctx);

        ctx.layout_rect = copy;

        perf().end("codex");
    }

    fn input(&mut self, state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        if self.creatures_button.input(&mut (), evt, ctx).is_break() {
            self.build_creatures(state, ctx);
            self.artifacts_button.set_selected(false);
            self.creatures_button.set_selected(true);
            self.sites_button.set_selected(false);
            self.quests_button.set_selected(false);
            return ControlFlow::Break(UIEvent::None)
        }
        if self.artifacts_button.input(&mut (), evt, ctx).is_break() {
            self.build_artifacts(state, ctx);
            self.creatures_button.set_selected(false);
            self.artifacts_button.set_selected(true);
            self.sites_button.set_selected(false);
            self.quests_button.set_selected(false);
            return ControlFlow::Break(UIEvent::None)

        }
        if self.sites_button.input(&mut (), evt, ctx).is_break() {
            self.build_sites(state, ctx);
            self.creatures_button.set_selected(false);
            self.artifacts_button.set_selected(false);
            self.sites_button.set_selected(true);
            self.quests_button.set_selected(false);
            return ControlFlow::Break(UIEvent::None)

        }
        if self.quests_button.input(&mut (), evt, ctx).is_break() {
            self.build_quests(state, ctx);
            self.creatures_button.set_selected(false);
            self.artifacts_button.set_selected(false);
            self.sites_button.set_selected(false);
            self.quests_button.set_selected(true);
            return ControlFlow::Break(UIEvent::None)

        }

        for (selection, button) in self.buttons.iter_mut() {
            if button.input(&mut (), evt, ctx).is_break() {
                self.selected = selection.clone();
                self.update_info(&state, ctx);
                return ControlFlow::Break(UIEvent::None)
            }
        }
        self.info_container.input(&mut (), evt, ctx)?;
        return ControlFlow::Continue(())
    }

}

#[derive(Clone)]
enum Selection {
    None,
    Creature(CreatureId),
    Artifact(ItemId),
    Site(SiteId),
    Quest(Quest)
}

fn quest_name(quest: &Quest) -> String {
    return match quest.objective {
        QuestObjective::KillBandits(_) => String::from("Kill bandits"),
        QuestObjective::KillWolves(_) => String::from("Kill wolves"),
        QuestObjective::KillVarningr(_) => String::from("Kill monster"),
    };   
}