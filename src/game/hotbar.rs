use std::{collections::HashSet, ops::ControlFlow};

use piston::Key;

use crate::{engine::{assets::assets, gui::{button::Button, layout_component::LayoutComponent, tooltip::{Tooltip, TooltipLine}, UINode}, input::InputEvent, render::RenderContext, COLOR_WHITE}, game::{actor::actor::Actor, inventory::inventory::EquipmentType}, resources::action::{Action, ActionEffect, ActionId, Affliction}, GameContext};

pub(crate) struct Hotbar {
    layout: LayoutComponent,
    background: String,
    available_actions: HashSet<ActionId>,
    equipped_actions: Vec<ActionId>,
    pub(crate) selected_action: Option<ActionId>,
    buttons: Vec<(ActionId, Button)>
}

impl Hotbar {
    pub(crate) fn new() -> Hotbar {
        let mut layout = LayoutComponent::new();
        layout.size([388., 26.]).anchor_bottom_center(0., 0.);
        Hotbar {
            layout,
            background: String::from("gui/hotbar/background.png"),
            available_actions: HashSet::new(),
            equipped_actions: Vec::new(),
            selected_action: None,
            buttons: Vec::new(),
        }
    }

    pub(crate) fn init(&mut self, actor: &Actor, ctx: &GameContext) {
        self.available_actions.insert(ctx.resources.actions.id_of("act:punch"));
        self.equip(actor, ctx);
    }

    pub(crate) fn equip(&mut self, actor: &Actor, ctx: &GameContext) {
        self.equipped_actions = Vec::new();
        for (_slot, equipped) in actor.inventory.all_equipped() {
            if let Some(action_provider) = &equipped.action_provider {
                self.equipped_actions.append(&mut action_provider.actions.clone());
            }
        }
        self.update_buttons(actor, &ctx);
    }

    fn update_buttons(&mut self, actor: &Actor, ctx: &GameContext) {
        self.buttons.clear();
        for action_id in self.available_actions.iter().chain(self.equipped_actions.iter()) {
            let action = ctx.resources.actions.get(action_id);
            self.buttons.push((*action_id, Button::image(action.icon.clone()).tooltip(Self::build_tooltip(&action, actor, ctx))))
        }
    }

    fn build_tooltip(action: &Action, actor: &Actor, ctx: &GameContext) -> Tooltip {
        let mut tooltip = Tooltip::new(&action.name);
        tooltip.add_line(TooltipLine::ApCost(action.ap_cost));
        tooltip.add_line(TooltipLine::StaminaCost(action.stamina_cost));
        for effect in action.effects.iter() {
            match &effect {
                ActionEffect::Damage { add_weapon, damage } => {
                    let mut damage = damage.clone();
                    if let Some(item) = actor.inventory.equipped(&EquipmentType::Hand) {
                        if *add_weapon {
                            damage = damage + item.total_damage(&ctx.resources.materials)
                        } else {
                            damage = damage + item.extra_damage(&ctx.resources.materials)
                        }
                    }
                    tooltip.add_line(TooltipLine::DamageRoll(damage));
                },
                ActionEffect::Inflicts { affliction } => {
                    let text = match affliction {
                        Affliction::Bleeding { duration } => format!("Target is bleeding for {duration} turns"),
                        Affliction::OnFire { duration } => format!("Target is on fire for {duration} turns"),
                        Affliction::Poisoned { duration } => format!("Target is poisoned for {duration} turns"),
                        Affliction::Stunned { duration } => format!("Target is stunned for {duration} turns"),
                        Affliction::MagicalHealing { duration } => format!("Target recovers health quickly for {duration} turns"),
                    };
                    tooltip.add_line(TooltipLine::Body(text));
                }
                _ => {}
            }
        }
        tooltip.add_line(TooltipLine::Body(action.description.clone()));
        return tooltip;
    }

    pub(crate) fn clear_selected(&mut self) {
        self.selected_action = None;
        for (_action_id, button) in self.buttons.iter_mut() {
            button.set_selected(false);
        }
    }
}

impl UINode for Hotbar {
    type State = Actor;
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        &mut self.layout
    }

    fn render(&mut self, state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let rect = self.layout.compute_layout_rect(ctx.layout_rect);
        ctx.image(&self.background, [rect[0] as i32, rect[1] as i32]);

        let copy = ctx.layout_rect;
        ctx.layout_rect = rect;
        ctx.layout_rect[0] += 74.;
        ctx.layout_rect[1] += 1.;

        for (id, button) in self.buttons.iter_mut() {
            ctx.layout_rect[0] += 24.;
            button.render(&(), ctx, game_ctx);
            let cooldown = state.cooldowns.iter().find(|cooldown| cooldown.0 == *id);
            if let Some(cooldown) = cooldown {
                ctx.text_shadow(&format!("{}", cooldown.1), assets().font_standard(), [ctx.layout_rect[0] as i32 + 8, ctx.layout_rect[1] as i32 + 16], &COLOR_WHITE);
            }
        }
        
        ctx.layout_rect = copy;

    }

    fn input(&mut self, _state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        let mut selected = None;
        match evt {
            InputEvent::Key { key: Key::Escape } => {
                if self.selected_action.is_some() {
                    self.clear_selected();
                    return ControlFlow::Break(());
                }
            },
            _ => ()
        }
        for (action_id, button) in self.buttons.iter_mut() {
            if let ControlFlow::Break(_) = button.input(&mut (), evt, ctx) {
                if self.selected_action.is_some_and(|id| &id == action_id) {
                    button.set_selected(false);
                    self.selected_action = None;
                    return ControlFlow::Break(())
                } else {
                    selected = Some(*action_id);
                }
            }
        }
        if let Some(action_id) = selected {
            self.selected_action = Some(action_id);
            for (b_action_id, button) in self.buttons.iter_mut() {
                button.set_selected(b_action_id == &action_id);
            }
            return ControlFlow::Break(())
        }
        ControlFlow::Continue(())
    }

}