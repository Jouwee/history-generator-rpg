use std::ops::ControlFlow;

use ai::AiSolver;
use chunk::Chunk;
use effect_layer::EffectLayer;
use game_context_menu::GameContextMenu;
use game_log::GameLog;
use graphics::draw_state::Blend;
use graphics::{image, DrawState, Image, Rectangle, Transformed};
use gui::character::character_dialog::CharacterDialog;
use gui::hud::HeadsUpDisplay;
use hotbar::Hotbar;
use map_modal::{MapModal, MapModalEvent};
use piston::{ButtonArgs, Key, MouseButton};
use player_pathing::PlayerPathing;
use crate::chunk_gen::chunk_generator::ChunkLayer;
use crate::commons::astar::AStar;
use crate::commons::interpolate::lerp;
use crate::engine::assets::assets;
use crate::engine::gui::button::Button;
use crate::engine::gui::dialog::DialogWrapper;
use crate::engine::gui::UINode;
use crate::engine::input::InputEvent as NewInputEvent;

use crate::engine::{Color, COLOR_WHITE};
use crate::game::chunk::{AiGroups, PLAYER_IDX};
use crate::game::console::Console;
use crate::game::gui::codex_dialog::CodexDialog;
use crate::resources::action::{ActionRunner, ActionArea};
use crate::world::creature::CreatureId;
use crate::world::world::World;
use crate::{engine::{audio::TrackMood, geometry::Coord2, gui::tooltip::TooltipOverlay, render::RenderContext, scene::{Scene, Update}}, GameContext};

pub(crate) mod actor;
pub(crate) mod ai;
pub(crate) mod chunk;
pub(crate) mod codex;
pub(crate) mod console;
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
    world_layer: ChunkLayer,
    pub(crate) chunk: Chunk,
    turn_mode: TurnMode,
    player_turn_timer: f64,
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
    console: Console,
    action_runner: ActionRunner,
    camera_offset: [f64; 2],
}

impl GameSceneState {
    pub(crate) fn new(mut world: World, world_pos: Coord2, chunk: Chunk) -> GameSceneState {
        let player_pathfinding = AStar::new(chunk.size, chunk.player().xy);

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
            world_layer: ChunkLayer::Surface,
            turn_mode: TurnMode::RealTime,
            player_turn_timer: 0.,
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
            console: Console::new(),
            action_runner: ActionRunner::new(),
            camera_offset: [0.; 2]
        }
    }

    fn save_creature_appearances(&mut self) {
        for npc in self.chunk.actors.iter() {
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
        if self.chunk.turn_controller.is_player_turn() {
            self.chunk.player_mut().ap.fill();
            self.chunk.player_mut().stamina.recover_turn();
            self.chunk.player_mut().hp.recover_turn();
        } else {
            let actor_ending = self.chunk.actors.get_mut(self.chunk.turn_controller.npc_idx()).unwrap();
            actor_ending.ap.fill();
            actor_ending.stamina.recover_turn();
            actor_ending.hp.recover_turn();
        }
        self.chunk.turn_controller.next_turn();
        if self.chunk.turn_controller.is_player_turn() {
            self.chunk.player_mut().start_of_round(&mut self.effect_layer);
        } else {       
            {
                let npc = self.chunk.actors.get_mut(self.chunk.turn_controller.npc_idx()).unwrap();
                npc.start_of_round(&mut self.effect_layer);
                if npc.hp.health_points() == 0. {
                    self.chunk.player_mut().add_xp(100);
                    self.chunk.remove_npc(self.chunk.turn_controller.npc_idx(), ctx);
                    self.next_turn(ctx);
                    return
                }
            }
            {
                let actor_idx = self.chunk.turn_controller.npc_idx();
                let actor = self.chunk.actors.get(actor_idx).unwrap();
                let state = AiSolver::check_state(&actor, &self.chunk);
                let actor = self.chunk.actors.get_mut(actor_idx).unwrap();
                actor.ai_state = state;
                let actor = self.chunk.actors.get(actor_idx).unwrap();
                let ai = AiSolver::choose_actions(&ctx.resources.actions, &actor, actor_idx, &self.chunk, ctx);
                let actor = self.chunk.actors.get_mut(actor_idx).unwrap();
                actor.ai = ai;
            }
        }
    }

    fn realtime_end_turn(&mut self, actor_idx: usize, ctx: &mut GameContext) {
        let actor = self.chunk.actors.get_mut(actor_idx).unwrap();
        actor.ap.fill();
        actor.stamina.recover_turn();
        actor.hp.recover_turn();
        actor.start_of_round(&mut self.effect_layer);
        let actor = self.chunk.actors.get(actor_idx).unwrap();
        let state = AiSolver::check_state(&actor, &self.chunk);
        let actor = self.chunk.actors.get_mut(actor_idx).unwrap();
        actor.ai_state = state;
        let actor = self.chunk.actors.get(actor_idx).unwrap();
        let ai = AiSolver::choose_actions(&ctx.resources.actions, &actor, self.chunk.turn_controller.npc_idx(), &self.chunk, ctx);
        let actor = self.chunk.actors.get_mut(actor_idx).unwrap();
        actor.ai = ai;
    }

    fn realtime_player_end_turn(&mut self) {
        let actor = self.chunk.player_mut();
        actor.ap.fill();
        actor.stamina.recover_turn();
        actor.hp.recover_turn();
        actor.start_of_round(&mut self.effect_layer);
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
        for npc in self.chunk.actors.iter() {
            if self.chunk.ai_groups.is_hostile(AiGroups::player(), npc.ai_group) {
                return false
            }
        }
        return true
    }

    fn move_to_chunk(&mut self, world_pos: Coord2, ctx: &mut GameContext) {
        // Move player to opposite side
        let mut player = self.chunk.player().clone();
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
        let chunk = Chunk::from_world_tile(&self.world, &ctx.resources, world_pos, self.world_layer, player);
        // Switcheroo
        self.world_pos = world_pos;
        self.chunk = chunk;
        // Re-init
        self.init(ctx);
    }

    fn move_to_layer(&mut self, layer: ChunkLayer, ctx: &mut GameContext) {
        // Creates the new chunk
        let mut chunk = Chunk::from_world_tile(&self.world, &ctx.resources, self.world_pos, layer, self.chunk.player().clone());
        // Finds the exit
        'outer: for x in 0..chunk.size.x() {
            for y in 0..chunk.size.y() {
                let pos = Coord2::xy(x as i32, y as i32);
                if chunk.map.get_object_idx(pos) == 17 {
                    chunk.player_mut().xy = pos + Coord2::xy(0, -1);
                    break 'outer;
                }
            }
        }
        // Switcheroo
        self.world_layer = layer;
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
        self.chunk.turn_controller.roll_initiative(self.chunk.actors.len());
        self.hotbar.init(&self.chunk.player(), ctx);
        self.game_context_menu.init(&(), ctx);
        if self.chunk.actors.iter().find(|actor| self.chunk.ai_groups.is_hostile(AiGroups::player(), actor.ai_group)).is_some() {
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
        let center = self.chunk.player().xy;
        self.camera_offset = [
            lerp(self.camera_offset[0], center.x as f64 * 24., 0.2),
            lerp(self.camera_offset[1], center.y as f64 * 24., 0.2),
        ];
        ctx.center_camera_on(self.camera_offset);

        self.chunk.render(ctx, game_ctx);

        if let Some(action_id) = &self.hotbar.selected_action {
            // TODO: Cleanup
            let action = game_ctx.resources.actions.get(action_id);
            let can_use = ActionRunner::can_use(action_id, action, PLAYER_IDX, self.cursor_pos, &self.chunk);
            let color = match can_use {
                Ok(_) => (COLOR_WHITE.alpha(0.2), COLOR_WHITE),
                Err(_) => (Color::from_hex("ff000030"), Color::from_hex("ff0000ff"))
            };
            if action.area != ActionArea::Target {
                for point in action.area.points(self.cursor_pos) {
                    ctx.rectangle_fill([point.x as f64 * 24., point.y as f64 * 24., 24., 24.], color.0);
                }
            }
            let image = assets().image("gui/cursor.png");
            let pos = [self.cursor_pos.x as f64 * 24., self.cursor_pos.y as f64 * 24.];
            let transform = ctx.context.transform.trans(pos[0], pos[1]);
            Image::new().color(color.1.f32_arr()).draw(&image.texture, &Default::default(), transform, ctx.gl);
            if let Err(msg) = can_use {
                ctx.text_shadow(&format!("{:?}", msg), game_ctx.assets.font_standard(), [pos[0] as i32, pos[1] as i32], &COLOR_WHITE);
            }
        } else {
            ctx.image("gui/cursor.png", [self.cursor_pos.x * 24, self.cursor_pos.y * 24]);
        }

        if self.hotbar.selected_action.is_none() {
            self.player_pathing.render(&self.turn_mode, self.chunk.player(), ctx);
        }

        // Effects
        self.effect_layer.render(ctx, game_ctx);

        if let ChunkLayer::Underground = self.world_layer {
            let draw_state = DrawState::new_alpha();
            let draw_state = draw_state.blend(Blend::Multiply);
            Rectangle::new(Color::from_hex("90a8b9ff").f32_arr()).draw(ctx.camera_rect, &draw_state, ctx.context.transform, ctx.gl);
        }

        // Back to screen space
        let _ = ctx.try_pop();

        // Vignette over game, under UI
        let vignette = assets().image("vignette.png");
        image(&vignette.texture, ctx.context.transform.scale(ctx.camera_rect[2] / vignette.size.0 as f64, ctx.camera_rect[3] / vignette.size.1 as f64), ctx.gl);

        // UI
        self.hotbar.render(&self.chunk.player, ctx, game_ctx);
        self.hud.render(self.chunk.player(), ctx, game_ctx);
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

        self.character_dialog.render(self.chunk.player_mut(), ctx, game_ctx);
        self.codex_dialog.render(&mut self.world, ctx, game_ctx);

        self.tooltip_overlay.render(&(), ctx, game_ctx); 
        self.game_context_menu.render(&(), ctx, game_ctx);

        self.console.render(ctx, game_ctx);
    }

    fn update(&mut self, update: &Update, ctx: &mut GameContext) {
        if let Some(map) = &mut self.map_modal {
            return map.update(update, ctx);
        }
        self.cursor_pos = Coord2::xy((update.mouse_pos_cam[0] / 24.) as i32, (update.mouse_pos_cam[1] / 24.) as i32);

        // TODO (OLaU4Dth): Ideally not every update
        if self.chunk.player().xy != *self.player_pathfinding.to() {
            self.player_pathfinding = AStar::new(self.chunk.size, self.chunk.player().xy);
        }
        if self.turn_mode == TurnMode::TurnBased {
            self.hud.preview_action_points(self.chunk.player(), self.player_pathing.get_preview_ap_cost());
        }

        self.hud.update(self.chunk.player(), update, ctx);
        if self.can_change_turn_mode() {
            match self.turn_mode {
                TurnMode::RealTime => self.button_toggle_turn_based.set_text("Trn"),
                TurnMode::TurnBased => self.button_toggle_turn_based.set_text("RT"),
            }
        }
        self.tooltip_overlay.update(&mut (), update, ctx); 
        self.effect_layer.update(update, ctx);

        let mut hostile = false;
        for npc in self.chunk.actors.iter_mut() {
            npc.update(update.delta_time);
            hostile = hostile || self.chunk.ai_groups.is_hostile(AiGroups::player(), npc.ai_group);
        }
        self.chunk.player_mut().update(update.delta_time);
        if hostile {
            self.set_turn_mode(TurnMode::TurnBased);
            ctx.audio.switch_music(TrackMood::Battle);
        } else {
            ctx.audio.switch_music(TrackMood::Regular);
        }

        // Check movement between chunks
        if self.chunk.player().xy.x <= 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(-1, 0), ctx);
            return
        }
        if self.chunk.player().xy.y <= 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(0, -1), ctx);
            return
        }
        if self.chunk.player().xy.x >= self.chunk.size.x() as i32 - 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(1, 0), ctx);
            return
        }
        if self.chunk.player().xy.y >= self.chunk.size.y() as i32 - 1 {
            self.move_to_chunk(self.world_pos + Coord2::xy(0, 1), ctx);
            return
        }
        // TODO: Resources
        if self.chunk.map.get_object_idx(self.chunk.player().xy) == 16 {
            self.move_to_layer(ChunkLayer::Underground, ctx);
        }
        if self.chunk.map.get_object_idx(self.chunk.player().xy) == 17 {
            self.move_to_layer(ChunkLayer::Surface, ctx);
        }

        self.action_runner.update(update, &mut self.chunk, &mut self.world, &mut self.effect_layer, &mut self.game_log, ctx);

        match self.turn_mode {
            TurnMode::TurnBased => {
                
                if self.chunk.turn_controller.is_player_turn() {
                    if self.player_pathing.is_running() {
                        self.player_pathing.update_running(&mut self.chunk, &mut self.world, &mut self.game_log, update, &mut self.action_runner, ctx);
                    }
                    return
                }
                let npc = self.chunk.actors.get_mut(self.chunk.turn_controller.npc_idx()).unwrap();

                if npc.ai.waiting_delay(update.delta_time) {
                    return
                }

                let next = npc.ai.next_action(&ctx.resources.actions);
                if let Some((action_id, action, cursor)) = next {
                    let v = self.action_runner.try_use(&action_id, action, self.chunk.turn_controller.npc_idx(), cursor, &mut self.chunk, &mut self.world, &mut self.game_log, ctx);
                    if let Err(v) = &v {
                        println!("AI tried to use action invalid: {:?}", v);
                    }
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
                    self.player_pathing.update_running(&mut self.chunk, &mut self.world, &mut self.game_log, update, &mut self.action_runner, ctx);
                }

                let mut end_turns_idxs = Vec::new();
                for (idx, npc) in self.chunk.actors.iter_mut().enumerate() {
                    if npc.ai.waiting_delay(update.delta_time) {
                        return
                    }

                    let next = npc.ai.next_action(&ctx.resources.actions);
                    if let Some((_, _, _)) = next {
                        // TODO: Borrow issues
                        // let v = self.action_runner.try_use(action, self.chunk.turn_controller.npc_idx(), cursor, &mut self.chunk, &mut self.world, &mut self.effect_layer, &mut self.game_log, ctx);
                        // if let Err(v) = &v {
                        //     println!("AI tried to use action invalid: {:?}", v);
                        // }
                    } else {
                        end_turns_idxs.push(idx);
                    }
                }
                for idx in end_turns_idxs {
                    self.realtime_end_turn(idx, ctx);
                }
                self.chunk.player_mut().ap.fill();
            }
        }

    }

    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<()> {

        if self.console.input(&mut self.chunk, &evt.evt, ctx).is_break() {
            return ControlFlow::Break(());
        }

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

        self.hotbar.input(&mut self.chunk.player, &evt.evt, ctx)?;
        self.hud.input(self.chunk.player(), &evt.evt, ctx);

        if self.character_dialog.input(self.chunk.player_mut(), &evt.evt, ctx).is_break() {
            self.hotbar.equip(&self.chunk.player(), ctx);
            return ControlFlow::Break(());
        }
        self.codex_dialog.input(&mut self.world, &evt.evt, ctx)?;


        if let ControlFlow::Break((cursor, action_id)) = self.game_context_menu.input(&mut (), &evt.evt, ctx) {
            let action = ctx.resources.actions.get(&action_id);

            let _ = self.action_runner.try_use(
                &action_id,
                action,
                PLAYER_IDX,
                cursor,
                &mut self.chunk,
                &mut self.world,
                &mut self.game_log,
                ctx
            );
            // Refreshes pathing
            self.player_pathfinding = AStar::new(self.chunk.size, self.chunk.player().xy);
            self.player_pathing.invalidate_pathing();

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
            d.init(self.chunk.player(), ctx);
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
                if !self.chunk.turn_controller.is_player_turn() {
                    return ControlFlow::Continue(());
                }
            },
            TurnMode::RealTime => {
                self.chunk.player_mut().ap.fill();
            }
        }

        if self.player_pathing.should_recompute_pathing(self.cursor_pos) {
            self.player_pathfinding.find_path(self.cursor_pos, |xy| self.chunk.astar_movement_cost(xy));
            self.player_pathing.set_preview(self.player_pathfinding.get_path(self.cursor_pos));
        }

        match evt.evt {
            NewInputEvent::Key { key: Key::Escape } => {
                self.hotbar.selected_action = None;
            },
            NewInputEvent::Key { key: Key::Space } => {
                if let TurnMode::TurnBased = self.turn_mode {
                    self.next_turn(ctx);
                }
            },
            NewInputEvent::Key { key: Key::M } => {
                let mut map = MapModal::new();
                map.init(&self.world, &self.world_pos);
                self.map_modal = Some(map);
            },
            NewInputEvent::Click { button: MouseButton::Right, pos } => {
                self.game_context_menu.show(PLAYER_IDX, self.cursor_pos, &mut self.chunk, ctx, pos);
            }
            NewInputEvent::Click { button: MouseButton::Left, pos: _ } => {
                if let Some(action_id) = &self.hotbar.selected_action {

                    let action = ctx.resources.actions.get(action_id);
                    let _ = self.action_runner.try_use(
                        action_id,
                        action,
                        PLAYER_IDX,
                        self.cursor_pos,
                        &mut self.chunk,
                        &mut self.world,
                        &mut self.game_log,
                        ctx
                    );
                    // Refreshes pathing
                    self.player_pathfinding = AStar::new(self.chunk.size, self.chunk.player().xy);
                    self.player_pathing.invalidate_pathing();
                } else {
                    if let Some(path) = &mut self.player_pathing.get_preview() {
                        self.player_pathing.start_running(path.clone());
                    }
                }
            }
            _ => (),
        }
        if self.turn_mode == TurnMode::RealTime {
            self.chunk.player_mut().ap.fill();
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
        // TODO: Attempt to subtract with overflow on loading a city
        return ((self.initiative[self.turn_idx] as i64 - 1) % self.initiative.len() as i64) as usize
    }

    pub(crate) fn next_turn(&mut self) {
        self.turn_idx = (self.turn_idx + 1) % self.initiative.len();
    }

}