use std::collections::HashSet;

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::{commons::id_vec::Id, engine::{gui::{button::{Button, ButtonEvent}, container::Container, hlist::HList, tooltip::{Tooltip, TooltipLine}, Anchor, GUINode, Position}, render::RenderContext, scene::Update, sprite::Sprite}, resources::action::{Action, ActionId, ActionType, Actions}, GameContext};

use super::{inventory::inventory::Inventory, InputEvent};

pub(crate) struct Hotbar {
    background: Texture,
    available_actions: HashSet<ActionId>,
    equipped_actions: Vec<ActionId>,
    pub(crate) selected_action: Option<ActionId>,
    action_buttons: HList
}

impl Hotbar {
    pub(crate) fn new() -> Hotbar {
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let background = ImageReader::open("assets/sprites/gui/hotbar/background.png").unwrap().decode().unwrap();
        Hotbar {
            background: Texture::from_image(&background.to_rgba8(), &settings),
            available_actions: HashSet::new(),
            equipped_actions: Vec::new(),
            selected_action: None,
            action_buttons: HList::new(Position::Anchored(Anchor::BottomCenter, 0., -24.))
        }
    }

    pub(crate) fn init(&mut self, inventory: &Inventory, ctx: &GameContext) {
        // TODO(vz4Z7ytt): Get from actor
        self.available_actions.insert(ctx.resources.actions.id_of("act:inspect"));
        self.available_actions.insert(ctx.resources.actions.id_of("act:pickup"));
        self.available_actions.insert(ctx.resources.actions.id_of("act:dig"));
        self.available_actions.insert(ctx.resources.actions.id_of("act:sleep"));
        self.available_actions.insert(ctx.resources.actions.id_of("act:punch"));
        self.equip(inventory, ctx);
    }

    pub(crate) fn equip(&mut self, inventory: &Inventory, ctx: &GameContext) {
        self.equipped_actions = Vec::new();
        if let Some(equipped) = inventory.equipped() {
            if let Some(action_provider) = &equipped.action_provider {
                self.equipped_actions = action_provider.actions.clone();
            }
        }
        self.update_buttons(&ctx.resources.actions);
    }

    fn update_buttons(&mut self, actions: &Actions) {
        self.action_buttons.clear();
        self.action_buttons.size = Some([128., 24.]);
        for action_id in self.available_actions.iter().chain(self.equipped_actions.iter()) {
            let action = actions.get(action_id);
            self.action_buttons.add_key(
                &format!("act_{}", action_id.as_usize()), 
                Button::new_bg(Sprite::new(action.icon.clone()).texture, Position::Auto)
                    .tooltip(Self::build_tooltip(action))
            );
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

    pub(crate) fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        // Background
        let center = ctx.layout_rect[2] / 2.;
        let base_pos = [center - 128., ctx.layout_rect[3] - 34.];
        ctx.texture_ref(&self.background, base_pos);

        self.action_buttons.render(ctx, game_ctx);
    }

    pub(crate) fn update(&mut self, update: &Update, ctx: &mut GameContext) {
        self.action_buttons.update(update, ctx);
    }

    pub(crate) fn input(&mut self, evt: &InputEvent, _ctx: &mut GameContext) {
        for action_id in self.available_actions.iter().chain(self.equipped_actions.iter()) {
            if let ButtonEvent::Click = self.action_buttons.get_mut::<Button>(&format!("act_{}", action_id.as_usize())).unwrap().event(evt) {
                self.selected_action = Some(*action_id);
            }
        }
    }
}

