use std::{collections::HashSet, ops::ControlFlow};

use crate::{engine::{asset::image::ImageAsset, gui::{button::Button, layout_component::LayoutComponent, tooltip::{Tooltip, TooltipLine}, UINode}, render::RenderContext}, resources::action::{Action, ActionId, ActionType, Actions}, GameContext};

use super::{inventory::inventory::Inventory};

pub(crate) struct Hotbar {
    layout: LayoutComponent,
    background: ImageAsset,
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
            background: ImageAsset::new("gui/hotbar/background.png"),
            available_actions: HashSet::new(),
            equipped_actions: Vec::new(),
            selected_action: None,
            buttons: Vec::new(),
        }
    }

    pub(crate) fn init(&mut self, inventory: &Inventory, ctx: &GameContext) {
        self.available_actions.insert(ctx.resources.actions.id_of("act:punch"));
        self.available_actions.insert(ctx.resources.actions.id_of("act:firebolt"));
        self.equip(inventory, ctx);
    }

    pub(crate) fn equip(&mut self, inventory: &Inventory, ctx: &GameContext) {
        self.equipped_actions = Vec::new();
        for (_slot, equipped) in inventory.all_equipped() {
            if let Some(action_provider) = &equipped.action_provider {
                self.equipped_actions = action_provider.actions.clone();
            }
        }
        self.update_buttons(&ctx.resources.actions);
    }

    fn update_buttons(&mut self, actions: &Actions) {
        self.buttons.clear();
        for action_id in self.available_actions.iter().chain(self.equipped_actions.iter()) {
            let action = actions.get(action_id);
            self.buttons.push((*action_id, Button::image(&action.icon).tooltip(Self::build_tooltip(action))))
        }
    }

    fn build_tooltip(action: &Action) -> Tooltip {
        let mut tooltip = Tooltip::new(action.name.clone());
        tooltip.add_line(TooltipLine::ApCost(action.ap_cost));
        tooltip.add_line(TooltipLine::StaminaCost(action.stamina_cost));
        match &action.action_type {
            ActionType::Targeted { damage, inflicts } => {
                if let Some(damage) = damage {
                    tooltip.add_line(TooltipLine::Damage(damage.clone()));
                }
                if let Some(inflicts) = inflicts {
                    tooltip.add_line(TooltipLine::Inflicts(inflicts.clone()));
                }
            }
            _ => ()
        };
        tooltip.add_line(TooltipLine::Body(action.description.clone()));
        return tooltip;
    }
}

impl UINode for Hotbar {
    type State = ();
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        &mut self.layout
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let rect = self.layout.compute_layout_rect(ctx.layout_rect);
        ctx.image(&self.background, [rect[0] as i32, rect[1] as i32], &mut game_ctx.assets);

        let copy = ctx.layout_rect;
        ctx.layout_rect = rect;
        ctx.layout_rect[0] += 62.;
        ctx.layout_rect[1] += 1.;

        for (_id, button) in self.buttons.iter_mut() {
            ctx.layout_rect[0] += 24.;
            button.render(&(), ctx, game_ctx);
        }
        
        ctx.layout_rect = copy;

    }

    fn input(&mut self, _state: &mut Self::State, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        let mut selected = None;
        for (action_id, button) in self.buttons.iter_mut() {
            if let ControlFlow::Break(()) = button.input(&mut (), evt, ctx) {
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