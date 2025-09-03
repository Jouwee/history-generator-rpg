use std::ops::ControlFlow;

use crate::{engine::{assets::assets, gui::{context_menu::{ContextMenu, ContextMenuModel}, layout_component::LayoutComponent, UIEvent, UINode}, scene::BusEvent, COLOR_WHITE}, game::actor::health_component::BodyPart, globals::perf::perf, loc, resources::resources::resources, world::item::Item, Actor, Color, EquipmentType, GameContext, InputEvent, RenderContext};

use super::{equipment_slot::EquipmentSlot, inventory_slot::InventorySlot};

const MENU_DROP: i32 = 0;
const MENU_CONSUME: i32 = 1;

pub(crate) struct CharacterDialog {
    layout: LayoutComponent,
    equipment_slot_hand: EquipmentSlot,
    equipment_slot_garment: EquipmentSlot,
    equipment_slot_inner_armor: EquipmentSlot,
    equipment_slot_head: EquipmentSlot,
    equipment_slot_legs: EquipmentSlot,
    equipment_slot_feet: EquipmentSlot,
    equipment_slot_trinket_1: EquipmentSlot,
    equipment_slot_trinket_2: EquipmentSlot,
    slots: Vec<InventorySlot>,
    cursor_pos: [i32; 2],
    context_menu: Option<(ContextMenu, ContextMenuModel, usize)>
}

impl CharacterDialog {
    
    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.anchor_center().size([360., 332.]).padding([8.; 4]);
        Self {
            layout,
            equipment_slot_hand: EquipmentSlot::new(EquipmentType::Hand, 0),
            equipment_slot_garment: EquipmentSlot::new(EquipmentType::TorsoGarment, 0),
            equipment_slot_inner_armor: EquipmentSlot::new(EquipmentType::TorsoInner, 0),
            equipment_slot_head: EquipmentSlot::new(EquipmentType::Head, 0),
            equipment_slot_legs: EquipmentSlot::new(EquipmentType::Legs, 0),
            equipment_slot_feet: EquipmentSlot::new(EquipmentType::Feet, 0),
            equipment_slot_trinket_1: EquipmentSlot::new(EquipmentType::Trinket, 0),
            equipment_slot_trinket_2: EquipmentSlot::new(EquipmentType::Trinket, 1),
            slots: Vec::new(),
            cursor_pos: [0; 2],
            context_menu: None,
        }
    }

    fn show_context_menu(&mut self, i: usize, item: &Option<Item>, pos: [f64; 2], ctx: &mut GameContext) {
        self.context_menu = item.as_ref().and_then(|item| {
            let mut menu = ContextMenu::new();
            let mut model = ContextMenuModel {
                items: vec!(
                    (MENU_DROP, loc!("item-ctx-menu-drop").clone()),
                )
            };

            let resources = resources();
            let blueprint = resources.item_blueprints.get(&item.blueprint_id);

            if blueprint.consumable.is_some() {
                model.items.push((MENU_CONSUME, loc!("item-ctx-menu-consume").clone()));
            }

            menu.layout_component().anchor_top_left(pos[0], pos[1]);
            menu.init(&model, ctx);
            Some((menu, model, i))
        });
    }

}

impl UINode for CharacterDialog {
    type State = Actor;
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, state: &Self::State, _game_ctx: &mut GameContext) {
        for _ in 0..state.inventory.container_len() {
            self.slots.push(InventorySlot::new());
        }
    }

    fn destroy(&mut self, state: &mut Self::State, game_ctx: &mut GameContext) {
        if let Some(item) = game_ctx.drag_item.take() {
            let _ = state.inventory.add(item);
        }
    }

    fn render(&mut self, actor: &Actor, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("character_dialog");

        let copy = ctx.layout_rect;


        ctx.layout_rect = self.layout.compute_inner_layout_rect(ctx.layout_rect);

        
        let copy2 = ctx.layout_rect;
        ctx.layout_rect = [
            ctx.layout_rect[0] + 8.,
            ctx.layout_rect[1] + 8.,
            158.,
            300.,
        ];
        ctx.rectangle_fill(ctx.layout_rect, &Color::from_hex("3a3c45"));
        
        let stats = actor.stats();

        let mut lines = Vec::new();
        lines.push(StatLine::Title(String::from("Health & Stamina")));
        lines.push(StatLine::Line(String::from("Health"), format!("{:.0} / {:.0}", actor.hp.health_points(), actor.hp.max_health_points())));
        lines.push(StatLine::Line(String::from("Action Points"), format!("{:.0} / {:.0}", actor.ap.action_points, actor.ap.max_action_points)));
        lines.push(StatLine::Line(String::from("Stamina"), format!("{:.0} / {:.0}", actor.stamina.stamina, actor.stamina.max_stamina)));

        lines.push(StatLine::Title(String::from("Condition")));
        lines.push(StatLine::Line(String::from("Head"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::Head).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Head protection"), format!("{}", stats.protection(&BodyPart::Head))));
        lines.push(StatLine::Line(String::from("Torso"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::Torso).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Torso protection"), format!("{}", stats.protection(&BodyPart::Torso))));
        lines.push(StatLine::Line(String::from("Left arm"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::LeftArm).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Left arm protection"), format!("{}", stats.protection(&BodyPart::LeftArm))));
        lines.push(StatLine::Line(String::from("Right arm"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::RightArm).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Right arm protection"), format!("{}", stats.protection(&BodyPart::RightArm))));
        lines.push(StatLine::Line(String::from("Left leg"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::LeftLeg).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Left leg protection"), format!("{}", stats.protection(&BodyPart::LeftLeg))));
        lines.push(StatLine::Line(String::from("Right leg"), format!("{:.0}%", actor.hp.body_part_condition(&BodyPart::RightLeg).unwrap().condition() * 100.)));
        lines.push(StatLine::Line(String::from("Right leg protection"), format!("{}", stats.protection(&BodyPart::RightLeg))));

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
                    ctx.text_shadow(title, assets().font_heading(), [layout[0], layout[1]], &COLOR_WHITE
                    );        
                    layout[1] += 16;
                },
                StatLine::Line(name, value) => {
                    ctx.text_shadow(name, assets().font_standard(), [layout[0], layout[1]], &Color::from_hex("7f839c"));
                    ctx.text_shadow(value, assets().font_standard(), [layout[0] + 103, layout[1]], &COLOR_WHITE);
                    layout[1] += 11;
                }
            }


        }

        ctx.layout_rect = copy2;

        let mut layout = [
            ctx.layout_rect[0] + 174.,
            ctx.layout_rect[1] + 8.,
            24.,
            24.
        ];

        ctx.text_shadow("Equipment", assets().font_heading(), [layout[0] as i32, layout[1] as i32 + 16], &COLOR_WHITE);
        layout[1] += 18.;

        let mut base = layout.clone();

        
        actor.render_layers([base[0] + 48., base[1] + 12.], ctx, game_ctx);

        ctx.text_shadow("Main hand", assets().font_standard(), [base[0] as i32, base[1] as i32 + 11], &Color::from_hex("7f839c"));
        base[1] += 12.;
        ctx.layout_rect = base;
        self.equipment_slot_hand.render(&actor.inventory, ctx, game_ctx);

        base[1] += 26.;

        ctx.text_shadow("Trinkets", assets().font_standard(), [base[0] as i32, base[1] as i32 + 11], &Color::from_hex("7f839c"));
        base[1] += 12.;
        ctx.layout_rect = base;
        self.equipment_slot_trinket_1.render(&actor.inventory, ctx, game_ctx);
        ctx.layout_rect[0] += 26.;
        self.equipment_slot_trinket_2.render(&actor.inventory, ctx, game_ctx);

        let mut base = layout.clone();
        base[0] += 112.;

        ctx.text_shadow("Head", assets().font_standard(), [base[0] as i32, base[1] as i32 + 11], &Color::from_hex("7f839c"));
        base[1] += 12.;
        ctx.layout_rect = base;
        self.equipment_slot_head.render(&actor.inventory, ctx, game_ctx);

        base[1] += 26.;

        ctx.text_shadow("Torso", assets().font_standard(), [base[0] as i32, base[1] as i32 + 11], &Color::from_hex("7f839c"));
        base[1] += 12.;
        ctx.layout_rect = base;
        self.equipment_slot_inner_armor.render(&actor.inventory, ctx, game_ctx);
        ctx.layout_rect[0] += 26.;
        self.equipment_slot_garment.render(&actor.inventory, ctx, game_ctx);

        base[1] += 26.;

        ctx.text_shadow("Legs", assets().font_standard(), [base[0] as i32, base[1] as i32 + 11], &Color::from_hex("7f839c"));
        base[1] += 12.;
        ctx.layout_rect = base;
        self.equipment_slot_legs.render(&actor.inventory, ctx, game_ctx);
        ctx.layout_rect[0] += 26.;
        self.equipment_slot_feet.render(&actor.inventory, ctx, game_ctx);

        base[0] = layout[0];
        base[1] += 32.;

        ctx.text_shadow("Inventory", assets().font_heading(), [base[0] as i32, base[1] as i32 + 16], &COLOR_WHITE);
        base[1] += 18.;

        for (i, slot) in self.slots.iter_mut().enumerate() {
            ctx.layout_rect = base;
            let x = i % 7;
            let y = i / 7;
            ctx.layout_rect[0] += x as f64 * 23.;
            ctx.layout_rect[1] += y as f64 * 23.;
            slot.render(&actor.inventory.item(i), ctx, game_ctx);
        }

        ctx.layout_rect = copy;

        if let Some((menu, model, _)) = &mut self.context_menu {
            menu.render(&model, ctx, game_ctx);
        }

        if let Some(item) = &game_ctx.drag_item {
            let texture = item.make_texture();
            ctx.texture_old(texture, [self.cursor_pos[0] as f64 - 12., self.cursor_pos[1] as f64 - 12.]);
        }

        perf().end("character_dialog");
    }

    fn input(&mut self, state: &mut Self::State, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {

        if let Some((menu, model, i)) = &mut self.context_menu {
            if let ControlFlow::Break((idu, _)) = menu.input(model, evt, ctx) {
                match idu {
                    MENU_DROP => ctx.event_bus.push(BusEvent::DropInventoryItem(*i)),
                    MENU_CONSUME => ctx.event_bus.push(BusEvent::ConsumeInventoryItem(*i)),
                    _ => (),
                }
                self.context_menu = None;
                return ControlFlow::Break(UIEvent::None)
            }
        }

        self.equipment_slot_hand.input(&mut state.inventory, evt, ctx)?;
        self.equipment_slot_garment.input(&mut state.inventory, evt, ctx)?;
        self.equipment_slot_inner_armor.input(&mut state.inventory, evt, ctx)?;
        self.equipment_slot_legs.input(&mut state.inventory, evt, ctx)?;
        self.equipment_slot_feet.input(&mut state.inventory, evt, ctx)?;
        self.equipment_slot_head.input(&mut state.inventory, evt, ctx)?;
        self.equipment_slot_trinket_1.input(&mut state.inventory, evt, ctx)?;
        self.equipment_slot_trinket_2.input(&mut state.inventory, evt, ctx)?;
        for (i, slot) in self.slots.iter_mut().enumerate() {
            match slot.input(&mut state.inventory.item_mut(i), evt, ctx) {
                ControlFlow::Break(UIEvent::ShowContextMenu(pos)) => {
                    self.show_context_menu(i, &state.inventory.item(i), pos, ctx);
                    return ControlFlow::Break(UIEvent::None);
                }
                other => other?
            }
        }
        match evt {
            InputEvent::MouseMove { pos } => self.cursor_pos = [pos[0] as i32, pos[1] as i32],
            InputEvent::Click { button: _, pos: _ } => self.context_menu = None,
            _ => ()
        }
        return ControlFlow::Continue(())
    }

}

enum StatLine {
    Title(String),
    Line(String, String)
}
