use std::collections::HashMap;

use action::{ActionRunner, ActionType};
use actor::ActorType;
use ai::AiSolver;
use chunk::Chunk;
use codex::{codex_dialog::CodexDialog, knowledge_codex::KnowledgeCodex};
use effect_layer::EffectLayer;
use hotbar::{Hotbar, HotbarState, NodeWithState};
use interact::interact_dialog::InteractDialog;
use inventory::character_dialog::{CharacterDialog, CharacterDialogOutput};
use map_modal::{MapModal, MapModalEvent};
use piston::{Button as Btn, ButtonArgs, ButtonState, Key};
use crate::engine::input::InputEvent as NewInputEvent;

use crate::{engine::{audio::TrackMood, geometry::Coord2, gui::{button::{Button, ButtonEvent}, tooltip::TooltipOverlay, Anchor, GUINode, Position}, render::RenderContext, scene::{Scene, Update}}, world::world::World, GameContext};

pub mod action;
pub mod actor;
pub mod ai;
pub mod chunk;
pub mod codex;
pub mod effect_layer;
pub mod hotbar;
pub mod interact;
pub mod inventory;
pub mod map_modal;
pub mod options;

pub trait Renderable {
    fn render(&self, ctx: &mut RenderContext, game_ctx: &mut GameContext);
}

pub struct InputEvent {
    pub mouse_pos_cam: [f64; 2],
    pub mouse_pos_gui: [f64; 2],
    pub button_args: ButtonArgs,
    pub evt: NewInputEvent
}

enum TurnMode {
    TurnBased,
    RealTime
}

pub struct GameSceneState {
    pub world: World,
    pub codex: KnowledgeCodex,
    pub world_pos: Coord2,
    pub chunk: Chunk,
    turn_mode: TurnMode,
    turn_controller: TurnController,
    button_codex: Button,
    button_inventory: Button,
    button_map: Button,
    button_end_turn: Button,
    button_toggle_turn_based: Button,
    hotbar: Hotbar,
    interact_dialog: InteractDialog,
    codex_dialog: CodexDialog,
    inventory_dialog: CharacterDialog,
    cursor_pos: Coord2,
    tooltip_overlay: TooltipOverlay,
    effect_layer: EffectLayer,
    map_modal: Option<MapModal>
}

impl GameSceneState {
    pub fn new(world: World, world_pos: Coord2, codex: KnowledgeCodex, chunk: Chunk) -> GameSceneState {
        GameSceneState {
            world,
            codex,
            chunk,
            world_pos,
            turn_mode: TurnMode::RealTime,
            turn_controller: TurnController::new(),
            hotbar: Hotbar::new(),
            button_inventory: Button::new("Character", Position::Anchored(Anchor::BottomLeft, 10.0, 32.0)),       
            button_codex: Button::new("Codex", Position::Anchored(Anchor::BottomLeft, 64.0, 32.0)),       
            button_map: Button::new("Map", Position::Anchored(Anchor::BottomCenter, -108.0, -24.0)),       
            button_end_turn: Button::new("End turn", Position::Anchored(Anchor::BottomCenter, 158.0, -32.0)),
            button_toggle_turn_based: Button::new("Enter turn-based mode", Position::Anchored(Anchor::BottomRight, 100.0, 32.0)),
            interact_dialog: InteractDialog::new(),
            codex_dialog: CodexDialog::new(),
            inventory_dialog: CharacterDialog::new(),
            cursor_pos: Coord2::xy(0, 0),
            tooltip_overlay: TooltipOverlay::new(),
            effect_layer: EffectLayer::new(),
            map_modal: None,
        }
    }

    fn save_creature_appearances(&mut self) {
        for npc in self.chunk.npcs.iter() {
            if let Some(id) = npc.person_id {
                let mut creature = self.world.people.get_mut(&id).unwrap();
                creature.appearance_hints = HashMap::new();
                for (k, v) in npc.sprite.map.iter() {
                    creature.appearance_hints.insert(k.clone(), v.0.clone());
                }
            }
        }
    }

    pub fn next_turn(&mut self, ctx: &mut GameContext) {
        if self.turn_controller.is_player_turn() {
            self.chunk.player.ap.fill();
        } else {
            let actor_ending = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
            actor_ending.ap.fill();
        }
        self.turn_controller.next_turn();
        if self.turn_controller.is_player_turn() {
            self.chunk.player.start_of_round(&mut self.effect_layer);
        } else {       
            {
                let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
                npc.start_of_round(&mut self.effect_layer);
                if npc.hp.health_points == 0. {
                    self.chunk.player.add_xp(100);
                    self.remove_npc(self.turn_controller.npc_idx(), ctx);
                    self.next_turn(ctx);
                    return
                }
            }
            {
                let npc = self.chunk.npcs.get(self.turn_controller.npc_idx()).unwrap();
                let ai = AiSolver::choose_actions(&ctx.resources.actions, &npc, &self.chunk, ctx);
                let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
                npc.ai = ai;
            }
        }
    }

    fn realtime_end_turn(&mut self, actor_idx: usize, ctx: &mut GameContext) {
        let actor = self.chunk.npcs.get_mut(actor_idx).unwrap();
        actor.ap.fill();
        actor.start_of_round(&mut self.effect_layer);
        let actor = self.chunk.npcs.get(actor_idx).unwrap();
        let ai = AiSolver::choose_actions(&ctx.resources.actions, &actor, &self.chunk, ctx);
        let actor = self.chunk.npcs.get_mut(actor_idx).unwrap();
        actor.ai = ai;
    }

    pub fn remove_npc(&mut self, i: usize, ctx: &mut GameContext) {
        let id;
        {
            let npc = self.chunk.npcs.get(i).unwrap();
            id = npc.person_id;
            for (_i, item, _equipped) in npc.inventory.iter() {
                self.chunk.items_on_ground.push((npc.xy, item.clone(), item.make_texture(&ctx.resources.materials)));
            }
        }
        self.chunk.npcs.remove(i);
        if let Some(id) = id {
            self.chunk.killed_people.push(id);
        }
        self.turn_controller.remove(i);
    }

    fn can_end_turn(&self) -> bool {
        if let TurnMode::TurnBased = self.turn_mode {
            return true
        }
        return false
    }

    fn can_change_turn_mode(&self) -> bool {
        if let TurnMode::RealTime = self.turn_mode {
            return true
        }
        for npc in self.chunk.npcs.iter() {
            if npc.actor_type == ActorType::Hostile {
                return false
            }
        }
        return true
    }

    fn move_to_chunk(&mut self, world_pos: Coord2, ctx: &mut GameContext) {
        // Move player to opposite side
        let mut player = self.chunk.player.clone();
        let offset = world_pos - self.world_pos;
        if offset.x < 0 {
            player.xy.x = self.chunk.size.x() as i32 - 2;
        }
        if offset.x > 0 {
            player.xy.x = 1;
        }
        if offset.y < 0 {
            player.xy.y = self.chunk.size.y() as i32 - 2;
        }
        if offset.y > 0 {
            player.xy.y = 1;
        }
        // Creates the new chunk
        // TODO: When out of bounds, make a special chunk gen
        let chunk = Chunk::from_world_tile(&self.world, &ctx.resources, world_pos, player);
        // Switcheroo
        self.world_pos = world_pos;
        self.chunk = chunk;
        // Re-init
        self.init(ctx);
    }

}

impl Scene for GameSceneState {
    fn init(&mut self, ctx: &mut GameContext) {
        self.save_creature_appearances();
        self.turn_controller.roll_initiative(self.chunk.npcs.len());
        self.hotbar.init(&self.chunk.player.inventory, ctx);
        if self.chunk.npcs.iter().find(|actor| actor.actor_type == ActorType::Hostile).is_some() {
            ctx.audio.switch_music(TrackMood::Battle);
        } else {
            ctx.audio.switch_music(TrackMood::Regular);
        }
    }

    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        ctx.pixel_art(2);
        ctx.push();
        if let Some(map) = &mut self.map_modal {
            return map.render(ctx, game_ctx);
        }
        // Game
        let center = self.chunk.player.xy;
        ctx.center_camera_on([center.x as f64 * 24., center.y as f64 * 24.]);
        self.chunk.render(ctx, game_ctx);

        if let Some(_) = self.hotbar.selected_action {
            ctx.image("cursor.png", [self.cursor_pos.x as f64 * 24., self.cursor_pos.y as f64 * 24.]);
        }
        // Effects
        self.effect_layer.render(ctx, game_ctx);
        // UI
        let _ = ctx.try_pop();
        self.hotbar.render(HotbarState::new(&self.chunk.player), ctx, game_ctx);
        self.button_codex.render(ctx, game_ctx);
        self.button_inventory.render(ctx, game_ctx);
        self.button_map.render(ctx, game_ctx);
        if self.can_end_turn() {
            self.button_end_turn.render(ctx, game_ctx);
        }
        if self.can_change_turn_mode() {
            self.button_toggle_turn_based.render(ctx, game_ctx);
        }
        self.interact_dialog.render(ctx, game_ctx);
        self.codex_dialog.render(ctx, game_ctx);
        self.inventory_dialog.render(ctx, game_ctx);       
        self.tooltip_overlay.render(ctx, game_ctx); 
    }

    fn update(&mut self, update: &Update, ctx: &mut GameContext) {
        if let Some(map) = &mut self.map_modal {
            return map.update(update, ctx);
        }

        self.hotbar.update(HotbarState::new(&self.chunk.player), update, ctx);
        self.button_codex.update(update, ctx);
        self.button_inventory.update(update, ctx);
        self.button_map.update(update, ctx);
        if self.can_end_turn() {
            self.button_end_turn.update(update, ctx);
        }
        if self.can_change_turn_mode() {
            match self.turn_mode {
                TurnMode::RealTime => self.button_toggle_turn_based.text("Enter turn-based mode"),
                TurnMode::TurnBased => self.button_toggle_turn_based.text("Exit turn-based mode"),
            }
            self.button_toggle_turn_based.update(update, ctx);
        }
        self.interact_dialog.update(update, ctx);
        self.codex_dialog.update(update, ctx);
        self.inventory_dialog.update(update, ctx);
        self.tooltip_overlay.update(update, ctx); 
        self.effect_layer.update(update, ctx);

        self.cursor_pos = Coord2::xy((update.mouse_pos_cam[0] / 24.) as i32, (update.mouse_pos_cam[1] / 24.) as i32);

        let mut hostile = false;
        for npc in self.chunk.npcs.iter_mut() {
            npc.update(update.delta_time);
            hostile = hostile || npc.actor_type == ActorType::Hostile;
        }
        self.chunk.player.update(update.delta_time);
        if hostile {
            self.turn_mode = TurnMode::TurnBased;
            ctx.audio.switch_music(TrackMood::Battle);
        } else {
            ctx.audio.switch_music(TrackMood::Regular);
        }

        // Check movement between chunks
        if self.chunk.player.xy.x == 0 {
            self.move_to_chunk(self.world_pos + Coord2::xy(-1, 0), ctx);
            return
        }
        if self.chunk.player.xy.y == 0 {
            self.move_to_chunk(self.world_pos + Coord2::xy(0, -1), ctx);
            return
        }
        if self.chunk.player.xy.x == self.chunk.size.x() as i32 - 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(1, 0), ctx);
            return
        }
        if self.chunk.player.xy.y == self.chunk.size.y() as i32 - 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(0, 1), ctx);
            return
        }

        match self.turn_mode {
            TurnMode::TurnBased => {
                if self.turn_controller.is_player_turn() {
                    return
                }
                let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();

                if npc.ai.waiting_delay(update.delta_time) {
                    return
                }

                let next = npc.ai.next_action(&ctx.resources.actions);
                if let Some(action) = next {
                    let _ = match action.action_type {
                        ActionType::Move { offset: _ } => ActionRunner::move_try_use(action, npc, &self.chunk.map, ctx, &self.chunk.player.xy),
                        ActionType::Targeted { damage: _, inflicts: _ } => ActionRunner::targeted_try_use(action, npc, &mut self.chunk.player, &mut self.effect_layer, ctx),
                        _ => true
                    };
                } else {
                    self.next_turn(ctx);
                }
            },
            TurnMode::RealTime => {
                let mut end_turns_idxs = Vec::new();
                for (idx, npc) in self.chunk.npcs.iter_mut().enumerate() {
                    if npc.ai.waiting_delay(update.delta_time) {
                        return
                    }

                    let next = npc.ai.next_action(&ctx.resources.actions);
                    if let Some(action) = next {
                        let _ = match action.action_type {
                            ActionType::Move { offset: _ } => ActionRunner::move_try_use(action, npc, &self.chunk.map, ctx, &self.chunk.player.xy),
                            ActionType::Targeted { damage: _, inflicts: _ } => ActionRunner::targeted_try_use(action, npc, &mut self.chunk.player, &mut self.effect_layer, ctx),
                            _ => true
                        };
                    } else {
                        end_turns_idxs.push(idx);
                    }
                }
                for idx in end_turns_idxs {
                    self.realtime_end_turn(idx, ctx);
                }
            }
        }

    }

    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) {
        if let Some(map) = &mut self.map_modal {
            if let MapModalEvent::Close = map.input(evt, ctx) {
                self.map_modal = None;
            }
            return
        }

        self.hotbar.input(HotbarState::new(&self.chunk.player), evt, ctx);
        self.interact_dialog.input_state(evt, &self.world, &ctx.resources, &mut self.codex);
        self.codex_dialog.input_state(evt, &self.world, &ctx.resources, &mut self.codex);
        let dialog_evt = self.inventory_dialog.input_state(evt, &mut self.chunk.player, &ctx.resources);
        match dialog_evt {
            CharacterDialogOutput::EquipmentChanged => self.hotbar.equip(&self.chunk.player.inventory, &ctx),
            CharacterDialogOutput::None => ()
        }

        if self.can_end_turn() {
            if let ButtonEvent::Click = self.button_end_turn.event(evt) {
                self.next_turn(ctx);
            }
        }
        if self.can_change_turn_mode() {
            if let ButtonEvent::Click = self.button_toggle_turn_based.event(evt) {
                match self.turn_mode {
                    TurnMode::RealTime => self.turn_mode = TurnMode::TurnBased,
                    TurnMode::TurnBased => self.turn_mode = TurnMode::RealTime,
                }
            }
        }

        if let ButtonEvent::Click = self.button_codex.event(evt) {
            self.codex_dialog.start_dialog();
            return;
        }

        if let ButtonEvent::Click = self.button_map.event(evt) {
            let mut map = MapModal::new();
            map.init(&self.world, &self.world_pos);
            self.map_modal = Some(map);
            return;
        }

        if let ButtonEvent::Click = self.button_inventory.event(evt) {
            self.inventory_dialog.start_dialog(&self.chunk.player, &ctx.resources);
            return;
        }

        match self.turn_mode {
            TurnMode::TurnBased => {
                if !self.turn_controller.is_player_turn() {
                    return
                }
            },
            TurnMode::RealTime => {
                self.chunk.player.ap.fill();
            }
        }
        if evt.button_args.state == ButtonState::Press {
            match evt.button_args.button {
                Btn::Keyboard(Key::Space) => {
                    if let TurnMode::TurnBased = self.turn_mode {
                        self.next_turn(ctx);
                    }
                },
                Btn::Keyboard(Key::M) => {
                    let mut map = MapModal::new();
                    map.init(&self.world, &self.world_pos);
                    self.map_modal = Some(map);
                },
                Btn::Keyboard(Key::Up) => {
                    let action = ctx.resources.actions.find("act:move_up");  
                    let xy = &self.chunk.player.xy.clone();
                    let _ = ActionRunner::move_try_use(action, &mut self.chunk.player, &self.chunk.map, ctx, xy);
                },
                Btn::Keyboard(Key::Down) => {
                    let action = ctx.resources.actions.find("act:move_down");  
                    let xy = &self.chunk.player.xy.clone();
                    let _ = ActionRunner::move_try_use(action, &mut self.chunk.player, &self.chunk.map, ctx, xy);
                },
                Btn::Keyboard(Key::Left) => {
                    let action = ctx.resources.actions.find("act:move_left");  
                    let xy = &self.chunk.player.xy.clone();
                    let _ = ActionRunner::move_try_use(action, &mut self.chunk.player, &self.chunk.map, ctx, xy);
                },
                Btn::Keyboard(Key::Right) => {
                    let action = ctx.resources.actions.find("act:move_right");  
                    let xy = &self.chunk.player.xy.clone();
                    let _ = ActionRunner::move_try_use(action, &mut self.chunk.player, &self.chunk.map, ctx, xy);
                },
                Btn::Mouse(_any) => {

                    if let Some(action_id) = &self.hotbar.selected_action {
                        let action = ctx.resources.actions.get(action_id);
                        if self.chunk.player.ap.can_use(action.ap_cost) {
                            match &action.action_type {
                                ActionType::Targeted { damage: _, inflicts: _ } => {
                                    let tile_pos = Coord2::xy(evt.mouse_pos_cam[0] as i32 / 24, evt.mouse_pos_cam[1] as i32 / 24);
                                    if tile_pos.dist_squared(&self.chunk.player.xy) < 3. {
                                        let target = self.chunk.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == tile_pos);
                                        if let Some((i, target)) = target {
                                            if ActionRunner::targeted_try_use(action, &mut self.chunk.player, target, &mut self.effect_layer, ctx) {
                                                println!("new hp: {}", target.hp.health_points);
                                                if target.hp.health_points == 0. {
                                                    self.chunk.player.add_xp(100);
                                                    self.remove_npc(i, ctx);
                                                }
                                                // Turn everyone hostile
                                                for p in self.chunk.npcs.iter_mut() {
                                                    p.actor_type = ActorType::Hostile;
                                                }
                                            }
                                        }
                                    }
                                },
                                ActionType::Talk => {
                                    let tile_pos = Coord2::xy(evt.mouse_pos_cam[0] as i32 / 24, evt.mouse_pos_cam[1] as i32 / 24);
                                    if tile_pos.dist_squared(&self.chunk.player.xy) < 3. {
                                        let target = self.chunk.npcs.iter_mut().find(|npc| npc.xy == tile_pos);
                                        if let Some(target) = target {
                                            self.interact_dialog.start_dialog(&self.world, target.person_id.unwrap());
                                        }
                                    }
                                }
                                ActionType::PickUp => {
                                    let tile_pos = Coord2::xy(evt.mouse_pos_cam[0] as i32 / 24, evt.mouse_pos_cam[1] as i32 / 24);
                                    if tile_pos.dist_squared(&self.chunk.player.xy) < 3. {
                                        let target = self.chunk.items_on_ground.iter_mut().enumerate().find(|(_i, (xy, _item, _tex))| *xy == tile_pos);
                                        if let Some((i, (_xy, item, _texture))) = target {
                                            self.chunk.player.inventory.add(item.clone());
                                            self.chunk.items_on_ground.remove(i);
                                        }
                                    }
                                }
                                ActionType::Sleep => {
                                    let tile_pos = Coord2::xy(evt.mouse_pos_cam[0] as i32 / 24, evt.mouse_pos_cam[1] as i32 / 24);
                                    // TODO: Bed
                                    if self.chunk.map.get_object_idx(tile_pos) == 3 {
                                        self.chunk.player.hp.refill();
                                    }
                                },
                                _ => ()
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

}

pub struct TurnController {
    turn_idx: usize,
    initiative: Vec<usize>
}

impl TurnController {

    pub fn new() -> TurnController {
        TurnController {
            initiative: vec!(),
            turn_idx: 0
        }
    }

    pub fn roll_initiative(&mut self, len: usize) {
        self.initiative = vec![0; len+1];
        for i in 0..len+1 {
            self.initiative[i] = i;
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.initiative.retain_mut(|i| {
            if *i == index + 1 {
                return false
            }
            if *i > index + 1 {
                *i = *i -1;
            }
            return true
        });
        self.turn_idx = self.turn_idx % self.initiative.len();
    }

    pub fn is_player_turn(&self) -> bool {
        return self.initiative[self.turn_idx] == 0
    }

    pub fn npc_idx(&self) -> usize {
        return self.initiative[self.turn_idx] - 1
    }

    pub fn next_turn(&mut self) {
        self.turn_idx = (self.turn_idx + 1) % self.initiative.len();
    }

}