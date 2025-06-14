use std::ops::ControlFlow;

use crate::{engine::gui::{button::Button, layout_component::LayoutComponent, UINode}, globals::perf::perf, world::{creature::CreatureId, world::World}, Color, GameContext, RenderContext};

pub(crate) struct CodexDialog {
    layout: LayoutComponent,
    buttons: Vec<(CreatureId, Button)>,
    selected: Selection
}

impl CodexDialog {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([400., 332.]).padding([8.; 4]);
        Self {
            layout,
            buttons: Vec::new(),
            selected: Selection::None
        }
    }

}

impl UINode for CodexDialog {
    type State = World;
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, state: &Self::State, game_ctx: &mut GameContext) {
        let mut y = 24.;
        for id in state.codex.creatures() {
            let creature = state.creatures.get(id);
            let mut button = Button::text(&creature.name(id, state, &game_ctx.resources));
            button.layout_component().anchor_top_left(24., y);
            self.buttons.push((*id, button));
            y += 24.;
        }
    }

    fn render(&mut self, state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("codex");

        let copy = ctx.layout_rect;


        ctx.layout_rect = self.layout.compute_inner_layout_rect(ctx);

        for (_id, button) in self.buttons.iter_mut() {
            button.render(&(), ctx, game_ctx);
        }

        if let Selection::Creature(creature_id) = &self.selected {
            let codex = state.codex.creature(creature_id).expect("Shouldn't have shown the button");
            let creature = state.creatures.get(creature_id);

            let mut layout = [ctx.layout_rect[0] as i32 + 130, ctx.layout_rect[1] as i32 + 16];

            if codex.know_name() {
                ctx.text_shadow(&creature.name(creature_id, state, &game_ctx.resources), game_ctx.assets.font_heading(), [layout[0], layout[1]], &Color::from_hex("ffffff"));            
            } else {
                ctx.text_shadow("?????", game_ctx.assets.font_heading(), [layout[0], layout[1]], &Color::from_hex("ffffff"));            
            }
            layout[1] += 16;

            let birth = match codex.know_birth() {
                true => format!("* {}-{}-{}", creature.birth.year(), creature.birth.month(), creature.birth.day()),
                false => String::from("* ?-?-?")
            };
            ctx.text_shadow(&birth, game_ctx.assets.font_standard(), [layout[0], layout[1]], &Color::from_hex("ffffff")); 

            let death = match codex.know_birth() {
                true => {
                    if let Some((date, _cause)) = creature.death {
                        format!("+ {}-{}-{}", date.year(), date.month(), date.day())
                    } else {
                        String::from("Alive")
                    }
                    
                },
                false => String::from("+ ?-?-?")
            };
            ctx.text_shadow(&death, game_ctx.assets.font_standard(), [layout[0] + 70, layout[1]], &Color::from_hex("ffffff")); 

            // ctx.text_shadow(value, game_ctx.assets.font_standard(), [layout[0] + 103, layout[1]], &Color::from_hex("ffffff"));
            // layout[1] += 11;

            // TODO(hu2htwck): Other info

        }

        ctx.layout_rect = copy;

        perf().end("codex");
    }

    fn input(&mut self, state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        for (id, button) in self.buttons.iter_mut() {
            if button.input(&mut (), evt, ctx).is_break() {
                println!("clicked id {:?}", id);
                self.selected = Selection::Creature(*id);
                return ControlFlow::Break(())
            }
        }
        return ControlFlow::Continue(())
    }

}

enum Selection {
    None,
    Creature(CreatureId)
}