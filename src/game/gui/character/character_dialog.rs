use crate::{engine::gui::new_ui::{InputResult, LayoutComponent, UINode}, game::actor::health_component::BodyPart, globals::perf::perf, Actor, Color, GameContext, RenderContext};

use super::{equipment_slot::EquipmentSlot, inventory_slot::InventorySlot};

pub(crate) struct CharacterDialog {
    layout: LayoutComponent,
    equipment_slot: EquipmentSlot,
    slots: Vec<InventorySlot>,
}

impl CharacterDialog {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([400., 282.]).padding([8.; 4]);
        Self {
            layout,
            equipment_slot: EquipmentSlot::new(),
            slots: Vec::new(),
        }
    }

}

impl UINode for CharacterDialog {
    type State = Actor;
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, state: &Self::State, _game_ctx: &mut GameContext) {
        for _ in 0..state.inventory.container_len() {
            self.slots.push(InventorySlot::new());
        }
    }

    fn render(&mut self, actor: &Actor, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("character_dialog");

        let copy = ctx.layout_rect;


        ctx.layout_rect = self.layout.compute_inner_layout_rect(ctx);

        
        let copy2 = ctx.layout_rect;
        ctx.layout_rect = [
            ctx.layout_rect[0] + 8.,
            ctx.layout_rect[1] + 8.,
            148.,
            250.,
        ];
        ctx.rectangle_fill(ctx.layout_rect, Color::from_hex("3a3c45"));
        
        let mut lines = Vec::new();
        lines.push(StatLine::Title(String::from("Health & Stamina")));
        lines.push(StatLine::Line(String::from("Health"), format!("{:.0} / {:.0}", actor.hp.health_points(), actor.hp.max_health_points())));
        lines.push(StatLine::Line(String::from("Action Points"), format!("{:.0} / {:.0}", actor.ap.action_points, actor.ap.max_action_points)));
        lines.push(StatLine::Line(String::from("Stamina"), format!("{:.0} / {:.0}", actor.stamina.stamina, actor.stamina.max_stamina)));

        lines.push(StatLine::Title(String::from("Condition")));
        lines.push(StatLine::Line(String::from("Head"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::Head).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Torso"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::Torso).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Left arm"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::LeftArm).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Right arm"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::RightArm).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Left leg"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::LeftLeg).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Right leg"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::RightLeg).unwrap().condition() * 100.)));

        let stats = actor.stats();
        lines.push(StatLine::Title(String::from("Combat")));
        lines.push(StatLine::Line(String::from("Crit chance"), format!("{:.2}%", stats.critical_hit_chance() * 100.)));
        lines.push(StatLine::Line(String::from("Crit damage"), format!("{:.2}", stats.critical_hit_multiplier())));
        lines.push(StatLine::Line(String::from("Dodge change"), format!("{:.2}%", stats.dodge_chance() * 100.)));
        lines.push(StatLine::Line(String::from("Movement AP mult"), format!("{:.2}", stats.walk_ap_multiplier())));

        let mut layout = [ctx.layout_rect[0] as i32 + 8, ctx.layout_rect[1] as i32 + 8];

        for line in lines.iter() {
            match line {
                StatLine::Title(title) => {
                    layout[1] += 11;
                    ctx.text_shadow(title, game_ctx.assets.font_heading(), [layout[0], layout[1]], &Color::from_hex("ffffff"));        
                    layout[1] += 16;
                },
                StatLine::Line(name, value) => {
                    ctx.text_shadow(name, game_ctx.assets.font_standard(), [layout[0], layout[1]], &Color::from_hex("ffffff"));
                    ctx.text_shadow(value, game_ctx.assets.font_standard(), [layout[0] + 93, layout[1]], &Color::from_hex("ffffff"));
                    layout[1] += 11;
                }
            }


        }

        ctx.layout_rect = copy2;

        let mut base = [
            ctx.layout_rect[0] + 164.,
            ctx.layout_rect[1] + 8.,
            24.,
            24.
        ];

        ctx.layout_rect = base;
        self.equipment_slot.render(&actor.inventory, ctx, game_ctx);

        base[1] += 32.;

        for (i, slot) in self.slots.iter_mut().enumerate() {
            ctx.layout_rect = base;
            let x = i % 7;
            let y = i / 7;
            ctx.layout_rect[0] += x as f64 * 23.;
            ctx.layout_rect[1] += y as f64 * 23.;
            slot.render(&actor.inventory.item(i), ctx, game_ctx);
        }

        ctx.layout_rect = copy;


        perf().end("character_dialog");
    }

    fn input(&mut self, state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> InputResult<Self::Input> {
        if self.equipment_slot.input(&mut state.inventory, evt, ctx).is_consumed() {
            return InputResult::Consume(())
        }
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if slot.input(&mut state.inventory.item_mut(i), evt, ctx).is_consumed() {
                return InputResult::Consume(())
            }
        }
        return InputResult::None
    }

}

enum StatLine {
    Title(String),
    Line(String, String)
}