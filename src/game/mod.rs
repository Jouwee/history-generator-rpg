use std::ops::ControlFlow;

use actor::actor::ActorType;
use ai::AiSolver;
use chunk::Chunk;
use effect_layer::EffectLayer;
use game_context_menu::GameContextMenu;
use game_log::GameLog;
use gui::character::character_dialog::CharacterDialog;
use gui::hud::HeadsUpDisplay;
use hotbar::Hotbar;
use map_modal::{MapModal, MapModalEvent};
use piston::{Button as Btn, ButtonArgs, ButtonState, Key, MouseButton};
use player_pathing::PlayerPathing;
use crate::commons::astar::AStar;
use crate::engine::asset::image::ImageAsset;
use crate::engine::gui::button::Button;
use crate::engine::gui::dialog::DialogWrapper;
use crate::engine::gui::UINode;
use crate::engine::input::InputEvent as NewInputEvent;

use crate::game::gui::codex_dialog::CodexDialog;
use crate::resources::action::{ActionParams, ActionRunner, ActionType, ActionUseParams};
use crate::world::creature::CreatureId;
use crate::world::world::World;
use crate::{engine::{audio::TrackMood, geometry::Coord2, gui::tooltip::TooltipOverlay, render::RenderContext, scene::{Scene, Update}}, GameContext};

pub(crate) mod actor;
pub(crate) mod ai;
pub(crate) mod chunk;
pub(crate) mod codex;
pub(crate) mod effect_layer;
pub(crate) mod factory;
pub(crate) mod game_log;
pub(crate) mod game_context_menu;
pub(crate) mod hotbar;
pub(crate) mod gui;
pub(crate) mod inventory;
pub(crate) mod map_modal;
pub(crate) mod options;
pub(crate) mod player_pathing;

const RT_TURN_TIME: f64 = 1.;

pub(crate) trait Renderable {
    fn render(&self, ctx: &mut RenderContext, game_ctx: &mut GameContext);
}

// TODO: Wtf is this?
pub(crate) struct InputEvent {
    pub(crate) mouse_pos_cam: [f64; 2],
    pub(crate) button_args: ButtonArgs,
    pub(crate) evt: NewInputEvent
}

#[derive(PartialEq, Eq)]
pub(crate) enum TurnMode {
    TurnBased,
    RealTime
}

pub(crate) struct GameSceneState {
    pub(crate) world: World,
    pub(crate) world_pos: Coord2,
    pub(crate) chunk: Chunk,
    turn_mode: TurnMode,
    player_turn_timer: f64,
    turn_controller: TurnController,
    button_inventory: Button,
    button_codex: Button,
    button_map: Button,
    button_end_turn: Button,
    button_toggle_turn_based: Button,
    hotbar: Hotbar,
    hud: HeadsUpDisplay,
    character_dialog: DialogWrapper<CharacterDialog>,
    codex_dialog: DialogWrapper<CodexDialog>,
    cursor_pos: Coord2,
    tooltip_overlay: TooltipOverlay,
    effect_layer: EffectLayer,
    game_context_menu: GameContextMenu,
    map_modal: Option<MapModal>,
    game_log: GameLog,
    player_pathing: PlayerPathing,
    player_pathfinding: AStar,
}

impl GameSceneState {
    pub(crate) fn new(mut world: World, world_pos: Coord2, chunk: Chunk) -> GameSceneState {
        let player_pathfinding = AStar::new(chunk.size, chunk.player.xy);

        let mut button_map = Button::text("Map");
        button_map.layout_component().anchor_bottom_center(-172.0, -1.0);
        let mut button_inventory = Button::text("Chr");
        button_inventory.layout_component().anchor_bottom_center(-147.0, -1.0);
        let mut button_codex = Button::text("Cdx");
        button_codex.layout_component().anchor_bottom_center(-122.0, -1.0);
        let mut button_end_turn = Button::text("Trn");
        button_end_turn.layout_component().anchor_bottom_center(147.0, -1.0);
        let mut button_toggle_turn_based = Button::text("Mod");
        button_toggle_turn_based.layout_component().anchor_bottom_center(172.0, -1.0);

        // TODO(hu2htwck): Test code
        let mut iter = world.creatures.iter_ids::<CreatureId>();
        let id1 = iter.next().unwrap().clone();
        let id2 = iter.next().unwrap().clone();
        let id3 = iter.next().unwrap().clone();
        let c = world.codex.creature_mut(&id1);
        c.add_name();
        c.add_appearance();
        c.add_birth();
        c.add_death();
        c.add_father();
        c.add_mother();
        for (i, event) in world.events.iter().enumerate() {
            if event.relates_to_creature(&id1) {
                c.add_event(i)
            }
        }

        let c = world.codex.creature_mut(&id2);
        c.add_name();
        for (i, event) in world.events.iter().enumerate() {
            if event.relates_to_creature(&id2) {
                c.add_event(i)
            }
        }
        let c = world.codex.creature_mut(&id3);
        c.add_birth();
        c.add_father();
        c.add_mother();
        for (i, event) in world.events.iter().enumerate() {
            if event.relates_to_creature(&id3) {
                c.add_event(i)
            }
        }

        GameSceneState {
            world,
            chunk,
            world_pos,
            turn_mode: TurnMode::RealTime,
            player_turn_timer: 0.,
            turn_controller: TurnController::new(),
            hotbar: Hotbar::new(),
            hud: HeadsUpDisplay::new(),
            button_map,
            button_inventory,
            button_codex,
            button_end_turn,
            button_toggle_turn_based,
            character_dialog: DialogWrapper::new(),
            codex_dialog: DialogWrapper::new(),
            cursor_pos: Coord2::xy(0, 0),
            tooltip_overlay: TooltipOverlay::new(),
            effect_layer: EffectLayer::new(),
            game_context_menu: GameContextMenu::new(),
            map_modal: None,
            game_log: GameLog::new(),
            player_pathing: PlayerPathing::new(),
            player_pathfinding,
        }
    }

    fn save_creature_appearances(&mut self) {
        for npc in self.chunk.npcs.iter() {
            if let Some(_id) = npc.creature_id {
                // TODO:
                // let mut creature = self.world.creatures.get_mut(&id).unwrap();
                // creature.appearance_hints = HashMap::new();
                // for (k, v) in npc.sprite.map.iter() {
                //     creature.appearance_hints.insert(k.clone(), v.0.clone());
                // }
            }
        }
    }

    pub(crate) fn next_turn(&mut self, ctx: &mut GameContext) {
        if self.turn_controller.is_player_turn() {
            self.chunk.player.ap.fill();
            self.chunk.player.stamina.recover_turn();
            self.chunk.player.hp.recover_turn();
        } else {
            let actor_ending = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
            actor_ending.ap.fill();
            actor_ending.stamina.recover_turn();
            actor_ending.hp.recover_turn();
        }
        self.turn_controller.next_turn();
        if self.turn_controller.is_player_turn() {
            self.chunk.player.start_of_round(&mut self.effect_layer);
        } else {       
            {
                let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
                npc.start_of_round(&mut self.effect_layer);
                if npc.hp.health_points() == 0. {
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
        actor.stamina.recover_turn();
        actor.hp.recover_turn();
        actor.start_of_round(&mut self.effect_layer);
        let actor = self.chunk.npcs.get(actor_idx).unwrap();
        let ai = AiSolver::choose_actions(&ctx.resources.actions, &actor, &self.chunk, ctx);
        let actor = self.chunk.npcs.get_mut(actor_idx).unwrap();
        actor.ai = ai;
    }

    fn realtime_player_end_turn(&mut self) {
        let actor = &mut self.chunk.player;
        actor.ap.fill();
        actor.stamina.recover_turn();
        actor.hp.recover_turn();
        actor.start_of_round(&mut self.effect_layer);
    }

    pub(crate) fn remove_npc(&mut self, i: usize, ctx: &mut GameContext) {
        let id;
        {
            let npc = self.chunk.npcs.get_mut(i).unwrap();
            id = npc.creature_id;
            for item in npc.inventory.take_all() {
                let texture = item.make_texture(&ctx.resources.materials);
                self.chunk.items_on_ground.push((npc.xy, item, texture));
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
            player.xy.x = 2;
        }
        if offset.y < 0 {
            player.xy.y = self.chunk.size.y() as i32 - 2;
        }
        if offset.y > 0 {
            player.xy.y = 2;
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

    fn set_turn_mode(&mut self, turn_mode: TurnMode) {
        match turn_mode {
            TurnMode::RealTime => {
                self.hud.clear_preview_action_points();
                self.player_turn_timer = 0.;
            },
            TurnMode::TurnBased => (),
        }
        self.turn_mode = turn_mode;
    }

}

impl Scene for GameSceneState {
    fn init(&mut self, ctx: &mut GameContext) {
        self.save_creature_appearances();
        self.turn_controller.roll_initiative(self.chunk.npcs.len());
        self.hotbar.init(&self.chunk.player.inventory, ctx);
        self.game_context_menu.init(&(), ctx);
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

        ctx.image(&ImageAsset::new("gui/cursor.png"), [self.cursor_pos.x * 24, self.cursor_pos.y * 24], &mut game_ctx.assets);

        if self.hotbar.selected_action.is_none() {
            self.player_pathing.render(&self.turn_mode, &self.chunk.player, ctx, game_ctx);
        }

        // Effects
        self.effect_layer.render(ctx, game_ctx);
        // UI
        let _ = ctx.try_pop();
        self.hotbar.render(&(), ctx, game_ctx);
        self.hud.render(&self.chunk.player, ctx, game_ctx);
        self.button_inventory.render(&(), ctx, game_ctx);
        self.button_codex.render(&(), ctx, game_ctx);
        self.button_map.render(&(), ctx, game_ctx);
        if self.can_end_turn() {
            self.button_end_turn.render(&(), ctx, game_ctx);
        }
        if self.can_change_turn_mode() {
            self.button_toggle_turn_based.render(&(), ctx, game_ctx);
        }
        self.game_log.render(ctx, game_ctx);

        self.character_dialog.render(&mut self.chunk.player, ctx, game_ctx);
        self.codex_dialog.render(&mut self.world, ctx, game_ctx);

        self.tooltip_overlay.render(&(), ctx, game_ctx); 
        self.game_context_menu.render(&(), ctx, game_ctx);
    }

    fn update(&mut self, update: &Update, ctx: &mut GameContext) {
        if let Some(map) = &mut self.map_modal {
            return map.update(update, ctx);
        }
        self.cursor_pos = Coord2::xy((update.mouse_pos_cam[0] / 24.) as i32, (update.mouse_pos_cam[1] / 24.) as i32);

        // TODO (OLaU4Dth): Ideally not every update
        if self.chunk.player.xy != *self.player_pathfinding.to() {
            self.player_pathfinding = AStar::new(self.chunk.size, self.chunk.player.xy);
        }
        if self.turn_mode == TurnMode::TurnBased {
            self.hud.preview_action_points(&self.chunk.player, self.player_pathing.get_preview_ap_cost());
        }

        self.hud.update(&self.chunk.player, update, ctx);
        if self.can_change_turn_mode() {
            match self.turn_mode {
                TurnMode::RealTime => self.button_toggle_turn_based.set_text("Trn"),
                TurnMode::TurnBased => self.button_toggle_turn_based.set_text("RT"),
            }
        }
        self.tooltip_overlay.update(&mut (), update, ctx); 
        self.effect_layer.update(update, ctx);

        let mut hostile = false;
        for npc in self.chunk.npcs.iter_mut() {
            npc.update(update.delta_time);
            hostile = hostile || npc.actor_type == ActorType::Hostile;
        }
        self.chunk.player.update(update.delta_time);
        if hostile {
            self.set_turn_mode(TurnMode::TurnBased);
            ctx.audio.switch_music(TrackMood::Battle);
        } else {
            ctx.audio.switch_music(TrackMood::Regular);
        }

        // Check movement between chunks
        if self.chunk.player.xy.x <= 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(-1, 0), ctx);
            return
        }
        if self.chunk.player.xy.y <= 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(0, -1), ctx);
            return
        }
        if self.chunk.player.xy.x >= self.chunk.size.x() as i32 - 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(1, 0), ctx);
            return
        }
        if self.chunk.player.xy.y >= self.chunk.size.y() as i32 - 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(0, 1), ctx);
            return
        }

        match self.turn_mode {
            TurnMode::TurnBased => {
                if self.turn_controller.is_player_turn() {
                    if self.player_pathing.is_running() {
                        self.player_pathing.update_running(&mut self.chunk.player, &self.chunk.map, update, ctx);
                    }
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
                        ActionType::Targeted { damage: _, inflicts: _ } => ActionRunner::targeted_try_use(action, npc, &mut self.chunk.player, &mut self.effect_layer, &mut self.game_log, &self.world, ctx),
                        _ => true
                    };
                } else {
                    self.next_turn(ctx);
                }
            },
            TurnMode::RealTime => {

                self.player_turn_timer += update.delta_time as f64;
                if self.player_turn_timer >= RT_TURN_TIME {
                    self.realtime_player_end_turn();
                    self.player_turn_timer -= RT_TURN_TIME;
                }

                if self.player_pathing.is_running() {
                    self.player_pathing.update_running(&mut self.chunk.player, &self.chunk.map, update, ctx);
                }

                let mut end_turns_idxs = Vec::new();
                for (idx, npc) in self.chunk.npcs.iter_mut().enumerate() {
                    if npc.ai.waiting_delay(update.delta_time) {
                        return
                    }

                    let next = npc.ai.next_action(&ctx.resources.actions);
                    if let Some(action) = next {
                        let _ = match action.action_type {
                            ActionType::Move { offset: _ } => ActionRunner::move_try_use(action, npc, &self.chunk.map, ctx, &self.chunk.player.xy),
                            ActionType::Targeted { damage: _, inflicts: _ } => ActionRunner::targeted_try_use(action, npc, &mut self.chunk.player, &mut self.effect_layer, &mut self.game_log, &self.world, ctx),
                            _ => true
                        };
                    } else {
                        end_turns_idxs.push(idx);
                    }
                }
                for idx in end_turns_idxs {
                    self.realtime_end_turn(idx, ctx);
                }
                self.chunk.player.ap.fill();
            }
        }

    }

    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<()> {
        if let Some(map) = &mut self.map_modal {
            match map.input(evt, ctx) {
                MapModalEvent::Close => self.map_modal = None,
                MapModalEvent::InstaTravelTo(coord) => {
                    self.move_to_chunk(coord, ctx);
                    self.map_modal = None;
                },
                MapModalEvent::None => ()
            }
            return ControlFlow::Continue(());
        }

        self.hotbar.input(&mut (), &evt.evt, ctx)?;
        self.hud.input(&self.chunk.player, &evt.evt, ctx);

        if self.character_dialog.input(&mut self.chunk.player, &evt.evt, ctx).is_break() {
            self.hotbar.equip(&self.chunk.player.inventory, ctx);
            return ControlFlow::Break(());
        }
        self.codex_dialog.input(&mut self.world, &evt.evt, ctx)?;


        if let ControlFlow::Break((cursor, action_id)) = self.game_context_menu.input(&mut (), &evt.evt, ctx) {
            let action = ctx.resources.actions.get(&action_id);

            let target = self.chunk.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == cursor);
            let item_on_ground = self.chunk.items_on_ground.iter().enumerate().find(|(_, (xy, _item, _tex))| *xy == cursor);
            let tile_metadata = self.chunk.tiles_metadata.get(&cursor).and_then(|m| Some(m));
            let object_tile = self.chunk.map.get_object_idx(cursor);

            let mut action_params = ActionUseParams {
                actor: &mut self.chunk.player,
                cursor_pos: cursor,
                target,
                item_on_ground: item_on_ground.and_then(|(i, t)| Some((i, &t.1))),
                tile_metadata,
                object_tile,
                world: &mut self.world
            };

            let result = ActionRunner::try_use(action, &mut action_params, &mut self.effect_layer, &mut self.game_log, ctx);
            if let Ok(side_effects) = result {
                for side_effect in side_effects {
                    side_effect.run(self, ctx);
                }
            }
            
            return ControlFlow::Break(());
        }


        if self.can_end_turn() {
            if self.button_end_turn.input(&mut (), &evt.evt, ctx).is_break() {
                self.next_turn(ctx);
                return ControlFlow::Break(());
            }
        }
        if self.can_change_turn_mode() {
            if self.button_toggle_turn_based.input(&mut (), &evt.evt, ctx).is_break() {
                match self.turn_mode {
                    TurnMode::RealTime => self.set_turn_mode(TurnMode::TurnBased),
                    TurnMode::TurnBased => self.set_turn_mode(TurnMode::RealTime),
                }
                return ControlFlow::Break(());
            }
        }

        if self.button_map.input(&mut (), &evt.evt, ctx).is_break() {
            let mut map = MapModal::new();
            map.init(&self.world, &self.world_pos);
            self.map_modal = Some(map);
            return ControlFlow::Break(());
        }

        if self.button_inventory.input(&mut (), &evt.evt, ctx).is_break() {

            // TODO(xYMCADko): init logic is weird
            let mut d = CharacterDialog::new();
            d.init(&self.chunk.player, ctx);
            self.character_dialog.show(d);

            return ControlFlow::Break(());
        }

        if self.button_codex.input(&mut (), &evt.evt, ctx).is_break() {

            // TODO(xYMCADko): init logic is weird
            let mut d = CodexDialog::new();
            d.init(&self.world, ctx);
            self.codex_dialog.show(d);

            return ControlFlow::Break(());
        }

        match self.turn_mode {
            TurnMode::TurnBased => {
                if !self.turn_controller.is_player_turn() {
                    return ControlFlow::Continue(());
                }
            },
            TurnMode::RealTime => {
                self.chunk.player.ap.fill();
            }
        }

        if self.player_pathing.recompute_pathing(self.cursor_pos) {
            self.player_pathfinding.find_path(self.cursor_pos, |xy| self.chunk.astar_movement_cost(xy));
            self.player_pathing.set_preview(self.player_pathfinding.get_path(self.cursor_pos));
        }

        if evt.button_args.state == ButtonState::Press {
            match evt.button_args.button {
                Btn::Keyboard(Key::Escape) => {
                    self.hotbar.selected_action = None;
                },
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
                _ => ()
            }
        }

        match evt.evt {
            NewInputEvent::Click { button: MouseButton::Right, pos } => {
                let tile_pos = Coord2::xy(evt.mouse_pos_cam[0] as i32 / 24, evt.mouse_pos_cam[1] as i32 / 24);
                let target = self.chunk.npcs.iter().find(|npc| npc.xy == tile_pos);
                let item_on_ground = self.chunk.items_on_ground.iter().find(|(xy, _item, _tex)| *xy == tile_pos);
                let tile_metadata = self.chunk.tiles_metadata.get(&tile_pos);
                let object_tile = self.chunk.map.get_object_idx(tile_pos);
                let action_params = ActionParams {
                    actor: &self.chunk.player,
                    cursor_pos: tile_pos,
                    target,
                    item_on_ground: item_on_ground.and_then(|i| Some(&i.1)),
                    tile_metadata,
                    object_tile
                };
                self.game_context_menu.show(&self.chunk.player, &action_params, ctx, pos, self.cursor_pos);
            }


            NewInputEvent::Click { button: MouseButton::Left, pos: _ } => {
                if let Some(action_id) = &self.hotbar.selected_action {

                    let target = self.chunk.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == self.cursor_pos);
                    let item_on_ground = self.chunk.items_on_ground.iter().enumerate().find(|(_, (xy, _item, _tex))| *xy == self.cursor_pos);
                    let tile_metadata = self.chunk.tiles_metadata.get(&self.cursor_pos).and_then(|m| Some(m));
                    let object_tile = self.chunk.map.get_object_idx(self.cursor_pos);

                    let mut action_params = ActionUseParams {
                        actor: &mut self.chunk.player,
                        cursor_pos: self.cursor_pos,
                        target,
                        item_on_ground: item_on_ground.and_then(|(i, t)| Some((i, &t.1))),
                        tile_metadata,
                        object_tile,
                        world: &mut self.world
                    };


                    let action = ctx.resources.actions.get(action_id);
                    let result = ActionRunner::try_use(action, &mut action_params, &mut self.effect_layer, &mut self.game_log, ctx);
                    if let Ok(side_effects) = result {
                        for side_effect in side_effects {
                            side_effect.run(self, ctx);
                        }
                    }
                } else {
                    if let Some(path) = &mut self.player_pathing.get_preview() {
                        self.player_pathing.start_running(path.clone());
                    }
                }
            }
            _ => (),
        }
        if self.turn_mode == TurnMode::RealTime {
            self.chunk.player.ap.fill();
        }
        return ControlFlow::Continue(())
    }

}

pub(crate) struct TurnController {
    turn_idx: usize,
    initiative: Vec<usize>
}

impl TurnController {

    pub(crate) fn new() -> TurnController {
        TurnController {
            initiative: vec!(),
            turn_idx: 0
        }
    }

    pub(crate) fn roll_initiative(&mut self, len: usize) {
        self.initiative = vec![0; len+1];
        for i in 0..len+1 {
            self.initiative[i] = i;
        }
    }

    pub(crate) fn remove(&mut self, index: usize) {
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

    pub(crate) fn is_player_turn(&self) -> bool {
        return self.initiative[self.turn_idx] == 0
    }

    pub(crate) fn npc_idx(&self) -> usize {
        return self.initiative[self.turn_idx] - 1
    }

    pub(crate) fn next_turn(&mut self) {
        self.turn_idx = (self.turn_idx + 1) % self.initiative.len();
    }

}