use std::ops::ControlFlow;

use ai::AiSolver;
use effect_layer::EffectLayer;
use engine::astar::AStar;
use game_context_menu::GameContextMenu;
use game_log::GameLog;
use graphics::draw_state::Blend;
use graphics::{image, DrawState, Image, Rectangle, Transformed};
use gui::character::character_dialog::CharacterDialog;
use gui::hud::HeadsUpDisplay;
use hotbar::Hotbar;
use map_modal::{MapModal, MapModalEvent};
use math::Vec2i;
use piston::{Key, MouseButton};
use player_pathing::PlayerPathing;
use serde::{Deserialize, Serialize};
use crate::commons::interpolate::lerp;
use crate::engine::assets::assets;
use crate::engine::audio::SoundEffect;
use crate::engine::gui::button::Button;
use crate::engine::gui::dialog::DialogWrapper;
use crate::engine::gui::tooltip::Tooltip;
use crate::engine::gui::UINode;
use crate::engine::input::InputEvent;

use crate::engine::scene::BusEvent;
use crate::engine::{Color, COLOR_BLACK, COLOR_WHITE};
use crate::game::ai::AiState;
use crate::game::chunk::{ChunkCoord, ChunkLayer};
use crate::game::codex::{QuestObjective, QuestStatus};
use crate::game::gui::character::ingame_menu::{InGameMenu, InGameMenuOption};
use crate::game::gui::chat_dialog::ChatDialog;
use crate::game::gui::codex_dialog::CodexDialog;
use crate::game::gui::death_dialog::DeathDialog;
use crate::game::gui::help_dialog::HelpDialog;
use crate::game::gui::inspect_dialog::InspectDialog;
use crate::game::gui::quest_complete_dialog::QuestCompleteDialog;
use crate::game::gui::time_widget::TimeWidget;
use crate::game::state::{AiGroups, GameState, PLAYER_IDX};
use crate::loadsave::SaveFile;
use crate::resources::action::{ActionRunner, ActionArea};
use crate::resources::resources::resources;
use crate::warn;
use crate::world::date::Duration;
use crate::world::history_sim::history_simulation::HistorySimulation;
use crate::world::site::SiteId;
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
pub(crate) mod map_component;
pub(crate) mod map_modal;
pub(crate) mod options;
pub(crate) mod player_pathing;
pub(crate) mod state;

const RT_TURN_TIME: f64 = 1.;

pub(crate) trait Renderable {
    fn render(&self, ctx: &mut RenderContext, game_ctx: &mut GameContext);
}

#[derive(PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum TurnMode {
    TurnBased,
    RealTime
}

pub(crate) struct GameSceneState {
    current_save_file: String,
    pub(crate) world: World,
    turn_mode: TurnMode,
    pub(crate) state: GameState,
    player_turn_timer: f64,
    button_inventory: Button,
    button_codex: Button,
    button_map: Button,
    button_help: Button,
    button_end_turn: Button,
    button_toggle_turn_based: Button,
    hotbar: Hotbar,
    hud: HeadsUpDisplay,
    time: TimeWidget,
    character_dialog: DialogWrapper<CharacterDialog>,
    codex_dialog: DialogWrapper<CodexDialog>,
    inspect_dialog: DialogWrapper<InspectDialog>,
    chat_dialog: DialogWrapper<ChatDialog>,
    quest_complete_dialog: DialogWrapper<QuestCompleteDialog>,
    death_dialog: DialogWrapper<DeathDialog>,
    help_dialog: DialogWrapper<HelpDialog>,
    ingame_menu: InGameMenu,
    cursor_pos: Coord2,
    tooltip_overlay: TooltipOverlay,
    effect_layer: EffectLayer,
    game_context_menu: GameContextMenu,
    map_modal: Option<MapModal>,
    game_log: GameLog,
    player_pathing: PlayerPathing,
    action_runner: ActionRunner,
    camera_offset: [f64; 2],
    shown_help: bool,
    /// Data for the sleep coroutine
    sleep_coroutine: Option<(f64, Duration, bool)>,
}

impl GameSceneState {
    pub(crate) fn new(world: World, save_file: String, state: GameState) -> GameSceneState {
        let mut button_map = Button::text("Map").tooltip(Tooltip::new("Show map (M)"));
        button_map.layout_component().anchor_bottom_center(-162.0, -1.0);
        let mut button_inventory = Button::text("Chr").tooltip(Tooltip::new("Character and inventory"));
        button_inventory.layout_component().anchor_bottom_center(-136.0, -1.0);
        let mut button_codex = Button::text("Cdx").tooltip(Tooltip::new("Codex"));
        button_codex.layout_component().anchor_bottom_center(-110.0, -1.0);
        let mut button_help = Button::text("?").tooltip(Tooltip::new("Help"));
        button_help.layout_component().anchor_bottom_center(132.0, -1.0);
        let mut button_end_turn = Button::text("Trn").tooltip(Tooltip::new("End turn (Space)"));
        button_end_turn.layout_component().anchor_bottom_center(157.0, -1.0);
        let mut button_toggle_turn_based = Button::text("Mod").tooltip(Tooltip::new("Togle Turn-based / Real time"));
        button_toggle_turn_based.layout_component().anchor_bottom_center(182.0, -1.0);

        GameSceneState {
            current_save_file: save_file,
            world,
            turn_mode: TurnMode::RealTime,
            state,
            player_turn_timer: 0.,
            hotbar: Hotbar::new(),
            hud: HeadsUpDisplay::new(),
            time: TimeWidget::new(),
            button_map,
            button_inventory,
            button_codex,
            button_help,
            button_end_turn,
            button_toggle_turn_based,
            character_dialog: DialogWrapper::new(),
            codex_dialog: DialogWrapper::new(),
            inspect_dialog: DialogWrapper::new(),
            chat_dialog: DialogWrapper::new(),
            quest_complete_dialog: DialogWrapper::new().hide_close_button(),
            death_dialog: DialogWrapper::new().hide_close_button(),
            help_dialog: DialogWrapper::new(),
            ingame_menu: InGameMenu::new(),
            cursor_pos: Coord2::xy(0, 0),
            tooltip_overlay: TooltipOverlay::new(),
            effect_layer: EffectLayer::new(),
            game_context_menu: GameContextMenu::new(),
            map_modal: None,
            game_log: GameLog::new(),
            player_pathing: PlayerPathing::new(),
            action_runner: ActionRunner::new(),
            camera_offset: [0.; 2],
            shown_help: false,
            sleep_coroutine: None,
        }
    }

    pub(crate) fn next_turn(&mut self, ctx: &mut GameContext) {
        if self.state.turn_controller.is_player_turn() {
            self.state.player_mut().ap.fill();
            self.state.player_mut().stamina.recover_turn();
            self.state.player_mut().hp.recover_turn();
        } else {
            let actor_ending = self.state.actors.get_mut(self.state.turn_controller.npc_idx()).unwrap();
            actor_ending.ap.fill();
            actor_ending.stamina.recover_turn();
            actor_ending.hp.recover_turn();
        }
        self.state.turn_controller.next_turn();
        if self.state.turn_controller.is_player_turn() {
            self.state.player_mut().start_of_round(&mut self.effect_layer);
        } else {       
            {
                let npc = self.state.actors.get_mut(self.state.turn_controller.npc_idx()).unwrap();
                npc.start_of_round(&mut self.effect_layer);
                if npc.hp.health_points() == 0. {
                    self.state.player_mut().add_xp(100);
                    self.state.remove_npc(self.state.turn_controller.npc_idx(), ctx);
                    self.next_turn(ctx);
                    return
                }
            }
            {
                let actor_idx = self.state.turn_controller.npc_idx();
                let actor = self.state.actors.get(actor_idx).unwrap();
                let state = AiSolver::check_state(&actor, &self.state);
                let actor = self.state.actors.get_mut(actor_idx).unwrap();
                actor.set_ai_state(state);
                let actor = self.state.actors.get(actor_idx).unwrap();
                let ai = AiSolver::choose_actions(&ctx.resources.actions, &actor, actor_idx, &self.state, ctx);
                let actor = self.state.actors.get_mut(actor_idx).unwrap();
                actor.ai = ai;
            }
        }
    }

    fn realtime_end_turn(&mut self, actor_idx: usize, ctx: &mut GameContext) {
        let actor = self.state.actors.get_mut(actor_idx).unwrap();
        actor.ap.fill();
        actor.stamina.recover_turn();
        actor.hp.recover_turn();
        actor.start_of_round(&mut self.effect_layer);
        let actor = self.state.actors.get(actor_idx).unwrap();
        let state = AiSolver::check_state(&actor, &self.state);
        let actor = self.state.actors.get_mut(actor_idx).unwrap();
        actor.set_ai_state(state);
        let actor = self.state.actors.get(actor_idx).unwrap();
        let ai = AiSolver::choose_actions(&ctx.resources.actions, &actor, self.state.turn_controller.npc_idx(), &self.state, ctx);
        let actor = self.state.actors.get_mut(actor_idx).unwrap();
        actor.ai = ai;
    }

    fn realtime_player_end_turn(&mut self) {
        let actor = self.state.player_mut();
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
        for npc in self.state.actors.iter() {
            if let AiState::Fight = npc.ai_state {
                return false
            }
        }
        return true
    }

    fn set_turn_mode(&mut self, turn_mode: TurnMode, ctx: &mut GameContext) {
        if turn_mode != self.turn_mode {
            match turn_mode {
                TurnMode::RealTime => {
                    ctx.audio.play_once(SoundEffect::new(vec!("game/exit_turn_based.mp3")));
                    self.hud.clear_preview_action_points();
                    self.player_turn_timer = 0.;
                },
                TurnMode::TurnBased => {
                    // Cancels running pathfinding
                    self.player_pathing.clear_running();
                    ctx.audio.play_once(SoundEffect::new(vec!("game/enter_turn_based.mp3")));
                },
            }
        }
        self.turn_mode = turn_mode;
    }

    fn simulate_time(&mut self, mut step: Duration) {
        let rng = self.world.rng().hash(self.world.date);
        let mut history_simulation = HistorySimulation::new(rng.into(), self.world.generation_parameters.clone());
        
        // Simulates at most 1 year per iteration
        while step > Duration::years(1) {
            history_simulation.simulate_step(Duration::years(1), &mut self.world);
            step = step - Duration::years(1);
        }
        history_simulation.simulate_step(step, &mut self.world);

        let save_file = SaveFile::new(self.current_save_file.clone());
        self.state.switch_chunk(self.state.coord.clone(), &save_file, &self.world);
    }

}

impl Scene for GameSceneState {
    type Input = ();

    fn init(&mut self, ctx: &mut GameContext) {
        self.state.turn_controller.roll_initiative(self.state.actors.len());
        self.hotbar.init(&self.state.player(), ctx);
        self.game_context_menu.init(&(), ctx);
        self.time.layout_component().anchor_top_right(16., 16.);

        if self.state.actors.iter().find(|actor| self.state.ai_groups.is_hostile(AiGroups::player(), actor.ai_group)).is_some() {
            ctx.audio.switch_music(TrackMood::Battle);
        } else {
            ctx.audio.switch_music(TrackMood::Regular);
        }
        if !self.shown_help {
            self.help_dialog.show(HelpDialog::new(), &mut (), ctx);
            self.shown_help = true;
        }
    }

    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        ctx.pixel_art(2);
        ctx.push();

        // Camera lerp
        let center = self.state.player().xy;
        self.camera_offset = [
            lerp(self.camera_offset[0], center.x() as f64 * 24., ctx.render_delta / 0.2),
            lerp(self.camera_offset[1], center.y() as f64 * 24., ctx.render_delta / 0.2),
        ];
        ctx.center_camera_on(self.camera_offset);

        self.state.render(ctx, game_ctx);

        if let Some(action_id) = &self.hotbar.selected_action {
            let action = game_ctx.resources.actions.get(action_id);
            let can_use = ActionRunner::can_use(action_id, &action, PLAYER_IDX, self.cursor_pos, &self.state);
            let color = match can_use {
                Ok(_) => (COLOR_WHITE.alpha(0.2), COLOR_WHITE),
                Err(_) => (Color::from_hex("ff000030"), Color::from_hex("ff0000ff"))
            };
            if action.area != ActionArea::Target {
                for point in action.area.points(self.cursor_pos) {
                    ctx.rectangle_fill([point.x as f64 * 24., point.y as f64 * 24., 24., 24.], &color.0);
                }
            }
            let image = assets().image("gui/cursor.png");
            let pos = [self.cursor_pos.x as f64 * 24., self.cursor_pos.y as f64 * 24.];
            let transform = ctx.context.transform.trans(pos[0], pos[1]);
            Image::new().color(color.1.f32_arr()).draw(&image.texture, &Default::default(), transform, ctx.gl);
            if let Err(msg) = can_use {
                ctx.text_shadow(&format!("{:?}", msg), assets().font_standard(), [pos[0] as i32, pos[1] as i32], &COLOR_WHITE);
            }
        } else {
            ctx.image("gui/cursor.png", [self.cursor_pos.x * 24, self.cursor_pos.y * 24]);
        }

        if self.hotbar.selected_action.is_none() {
            self.player_pathing.render(&self.turn_mode, self.state.player(), ctx);
        }

        // Effects
        self.effect_layer.render(ctx);

        if let ChunkLayer::Underground = self.state.coord.layer {
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
        self.hotbar.render(&self.state.player, ctx, game_ctx);
        self.hud.render(self.state.player(), ctx, game_ctx);
        self.time.render(&(), ctx, game_ctx);
        self.button_inventory.render(&(), ctx, game_ctx);
        self.button_codex.render(&(), ctx, game_ctx);
        self.button_map.render(&(), ctx, game_ctx);
        self.button_help.render(&(), ctx, game_ctx);
        if self.can_end_turn() {
            self.button_end_turn.render(&(), ctx, game_ctx);
        }
        if self.can_change_turn_mode() {
            self.button_toggle_turn_based.render(&(), ctx, game_ctx);
        }
        self.game_log.render(ctx);

        self.character_dialog.render(self.state.player_mut(), ctx, game_ctx);
        self.codex_dialog.render(&mut self.world, ctx, game_ctx);
        self.inspect_dialog.render(&mut self.world, ctx, game_ctx);
        self.chat_dialog.render(&mut self.world, ctx, game_ctx);
        self.quest_complete_dialog.render(&mut self.world, ctx, game_ctx);
        self.death_dialog.render(&mut (), ctx, game_ctx);
        self.help_dialog.render(&mut (), ctx, game_ctx);
        self.ingame_menu.render(&mut (), ctx, game_ctx);

        if let Some(map) = &mut self.map_modal {
            map.render(ctx, game_ctx);
        }

        self.tooltip_overlay.render(&(), ctx, game_ctx); 
        self.game_context_menu.render(&(), ctx, game_ctx);

        if let Some((timer, _, _)) = self.sleep_coroutine {
            let alpha;
            if timer < 0.5 {
                alpha = lerp(0., 1., timer / 0.5);
            } else {
                alpha = lerp(1., 0., (timer - 0.5) / 0.5);
            }
            ctx.rectangle_fill(ctx.layout_rect, &COLOR_BLACK.alpha(alpha as f32));
        }

    }

    fn update(&mut self, update: &Update, ctx: &mut GameContext) {
        self.time.set_time(&self.world.date);

        if let Some((mut timer, duration, mut updated)) = self.sleep_coroutine.take() {
            timer += update.delta_time;
            if timer > 0.5 && !updated {
                self.simulate_time(duration);
                updated = true;
            }
            if timer < 1. || !updated {
                self.sleep_coroutine = Some((timer, duration, updated));
            }
            return;
        }

        if let Some(map) = &mut self.map_modal {
            return map.update(update, ctx);
        }

        // Pauses the game while the menu is open
        if self.ingame_menu.is_visible() {
            return;
        }

        // TODO: Should not be done in update. Input doesn't have "mouse pos"
        self.cursor_pos = Coord2::xy((update.mouse_pos_cam[0] / 24.) as i32, (update.mouse_pos_cam[1] / 24.) as i32);

        if self.turn_mode == TurnMode::TurnBased {
            self.hud.preview_action_points(self.state.player(), self.player_pathing.get_preview_ap_cost());
        }

        self.hud.update(self.state.player(), update, ctx);
        if self.can_change_turn_mode() {
            match self.turn_mode {
                TurnMode::RealTime => self.button_toggle_turn_based.set_text("Trn"),
                TurnMode::TurnBased => self.button_toggle_turn_based.set_text("RT"),
            }
        }
        self.tooltip_overlay.update(&mut (), update, ctx); 
        self.effect_layer.update(update, ctx);

        let mut in_fight = false;
        for actor in self.state.actors.iter_mut() {
            actor.update(update.delta_time);
            if let AiState::Fight = actor.ai_state {
                in_fight = true;
            }
        }
        self.state.player_mut().update(update.delta_time);
        if in_fight {
            self.set_turn_mode(TurnMode::TurnBased, ctx);
            ctx.audio.switch_music(TrackMood::Battle);
        } else {
            ctx.audio.switch_music(TrackMood::Regular);
        }

        let save_file = SaveFile::new(self.current_save_file.clone());

        // Check movement between chunks
        if self.state.player().xy.x() <= 1 {
            self.state.switch_chunk(ChunkCoord::new(self.state.coord.xy + Vec2i(-1, 0), self.state.coord.layer), &save_file, &self.world);
            return
        }
        if self.state.player().xy.y() <= 1 {
            self.state.switch_chunk(ChunkCoord::new(self.state.coord.xy + Vec2i(0, -1), self.state.coord.layer), &save_file, &self.world);
            return
        }
        if self.state.player().xy.x() >= self.state.chunk.size.x() as i32 - 2 {
            self.state.switch_chunk(ChunkCoord::new(self.state.coord.xy + Vec2i(1, 0), self.state.coord.layer), &save_file, &self.world);
            return
        }
        if self.state.player().xy.y() >= self.state.chunk.size.y() as i32 - 2 {
            self.state.switch_chunk(ChunkCoord::new(self.state.coord.xy + Vec2i(0, 1), self.state.coord.layer), &save_file, &self.world);
            return
        }
        let resources = resources();
        if self.state.chunk.get_object_id(self.state.player().xy.into()).map(|id| id == resources.object_tiles.id_of("obj:ladder_down")).unwrap_or(false) {
            self.state.switch_chunk(ChunkCoord::new(self.state.coord.xy, ChunkLayer::Underground), &save_file, &self.world);
        }
        if self.state.chunk.get_object_id(self.state.player().xy.into()).map(|id| id == resources.object_tiles.id_of("obj:ladder_up")).unwrap_or(false) {
            self.state.switch_chunk(ChunkCoord::new(self.state.coord.xy, ChunkLayer::Surface), &save_file, &self.world);
        }

        self.action_runner.update(update, &mut self.state, &mut self.world, &mut self.effect_layer, &mut self.game_log, ctx);

        match self.turn_mode {
            TurnMode::TurnBased => {
                
                if self.state.turn_controller.is_player_turn() {
                    if self.player_pathing.is_running() {
                        self.player_pathing.update_running(&mut self.state, &mut self.world, &mut self.game_log, update, &mut self.action_runner, ctx);
                    }
                    return
                }
                let npc = self.state.actors.get_mut(self.state.turn_controller.npc_idx()).unwrap();

                if npc.ai.waiting_delay(update.delta_time) {
                    return
                }

                let next = npc.ai.next_action(&ctx.resources.actions);
                if let Some((action_id, cursor)) = next {
                    let action = ctx.resources.actions.get(&action_id);
                    let v = self.action_runner.try_use(&action_id, &action, self.state.turn_controller.npc_idx(), cursor, &mut self.state, &mut self.world, &mut self.game_log, ctx);
                    if let Err(v) = &v {
                        warn!("AI tried to use action invalid: {:?}", v);
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
                    self.player_pathing.update_running(&mut self.state, &mut self.world, &mut self.game_log, update, &mut self.action_runner, ctx);
                }

                let mut end_turns_idxs = Vec::new();
                for (idx, npc) in self.state.actors.iter_mut().enumerate() {
                    if npc.ai.waiting_delay(update.delta_time) {
                        return
                    }

                    let next = npc.ai.next_action(&ctx.resources.actions);
                    if let Some((_, _)) = next {
                        // TODO: Borrow issues
                        // let action = ctx.resources.actions.get(&action_id);
                        // let v = self.action_runner.try_use(action, self.chunk.turn_controller.npc_idx(), cursor, &mut self.chunk, &mut self.world, &mut self.effect_layer, &mut self.game_log, ctx);
                        // if let Err(v) = &v {
                        //     warn!("AI tried to use action invalid: {:?}", v);
                        // }
                    } else {
                        end_turns_idxs.push(idx);
                    }
                }
                for idx in end_turns_idxs {
                    self.realtime_end_turn(idx, ctx);
                }
                self.state.player_mut().ap.fill();
            }
        }

    }

    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<()> {

        if self.death_dialog.is_visible() {
            self.death_dialog.input(&mut (), &evt, ctx)?;
            // Returns to avoid any other component receiving events
            return ControlFlow::Continue(());
        }

        if let Some(map) = &mut self.map_modal {
            match map.input(&evt, ctx) {
                ControlFlow::Break(MapModalEvent::Close) => self.map_modal = None,
                ControlFlow::Break(MapModalEvent::InstaTravelTo(coord)) => {
                    let save_file = SaveFile::new(self.current_save_file.clone());
                    self.state.switch_chunk(ChunkCoord::new(coord.to_vec2i(), ChunkLayer::Surface), &save_file, &self.world);
                    self.map_modal = None;
                },
                _ => ()
            }
            return ControlFlow::Continue(());
        }

        if self.character_dialog.input(self.state.player_mut(), &evt, ctx).is_break() {
            self.hotbar.equip(&self.state.player(), ctx);
            return ControlFlow::Break(());
        }
        self.codex_dialog.input(&mut self.world, &evt, ctx)?;
        self.inspect_dialog.input(&mut self.world, &evt, ctx)?;
        self.chat_dialog.input(&mut self.world, &evt, ctx)?;
        self.quest_complete_dialog.input(&mut self.world, &evt, ctx)?;
        self.help_dialog.input(&mut (), &evt, ctx)?;
        
        match self.ingame_menu.input(&mut (), &evt, ctx) {
            ControlFlow::Break(InGameMenuOption::None) => return ControlFlow::Break(()),
            ControlFlow::Break(InGameMenuOption::SaveGame) => {
                let load_save_manager = SaveFile::new(self.current_save_file.clone());
                load_save_manager.save_world(&self.world).unwrap();
                load_save_manager.save_game_state(&self.state).unwrap();
                load_save_manager.save_chunk(&self.state.chunk).unwrap();
            
                return ControlFlow::Break(())
            },
            ControlFlow::Continue(()) => (),
        }

        self.hotbar.input(&mut self.state.player, &evt, ctx)?;
        self.hud.input(self.state.player(), &evt, ctx);

        if let ControlFlow::Break((cursor, action_id)) = self.game_context_menu.input(&mut (), &evt, ctx) {
            let action = ctx.resources.actions.get(&action_id);

            let _ = self.action_runner.try_use(
                &action_id,
                &action,
                PLAYER_IDX,
                cursor,
                &mut self.state,
                &mut self.world,
                &mut self.game_log,
                ctx
            );
            return ControlFlow::Break(());
        }


        if self.can_end_turn() {
            if self.button_end_turn.input(&mut (), &evt, ctx).is_break() {
                self.next_turn(ctx);
                return ControlFlow::Break(());
            }
        }
        if self.can_change_turn_mode() {
            if self.button_toggle_turn_based.input(&mut (), &evt, ctx).is_break() {
                match self.turn_mode {
                    TurnMode::RealTime => self.set_turn_mode(TurnMode::TurnBased, ctx),
                    TurnMode::TurnBased => self.set_turn_mode(TurnMode::RealTime, ctx),
                }
                return ControlFlow::Break(());
            }
        }

        if self.button_map.input(&mut (), &evt, ctx).is_break() {
            let mut map = MapModal::new();
            map.init(&self.world, &self.state.coord.xy.into());
            self.map_modal = Some(map);
            return ControlFlow::Break(());
        }

        if self.button_inventory.input(&mut (), &evt, ctx).is_break() {
            self.character_dialog.show(CharacterDialog::new(), self.state.player(), ctx);
            return ControlFlow::Break(());
        }

        if self.button_codex.input(&mut (), &evt, ctx).is_break() {
            self.codex_dialog.show(CodexDialog::new(), &self.world, ctx);
            return ControlFlow::Break(());
        }

        if self.button_help.input(&mut (), &evt, ctx).is_break() {
            self.help_dialog.show(HelpDialog::new(), &(), ctx);
            return ControlFlow::Break(());
        }

        match self.turn_mode {
            TurnMode::TurnBased => {
                if !self.state.turn_controller.is_player_turn() {
                    return ControlFlow::Continue(());
                }
            },
            TurnMode::RealTime => {
                self.state.player_mut().ap.fill();
            }
        }

        if self.player_pathing.should_recompute_pathing(self.cursor_pos.to_vec2i()) {
            let mut player_pathfinding = AStar::new(self.state.chunk.size.vec2i(), self.state.player().xy);
            player_pathfinding.find_path(self.cursor_pos.to_vec2i(), |xy| self.state.astar_movement_cost(xy.into()));
            self.player_pathing.set_preview(self.cursor_pos.to_vec2i(), player_pathfinding.get_path(self.cursor_pos.to_vec2i()));
        }

        match evt {
            InputEvent::Key { key: Key::Escape } => {
                self.ingame_menu.show();
            },
            InputEvent::Key { key: Key::Space } => {
                if let TurnMode::TurnBased = self.turn_mode {
                    self.next_turn(ctx);
                }
            },
            InputEvent::Key { key: Key::M } => {
                let mut map = MapModal::new();
                map.init(&self.world, &self.state.coord.xy.into());
                self.map_modal = Some(map);
            },
            InputEvent::Click { button: MouseButton::Right, pos } => {
                self.game_context_menu.show(PLAYER_IDX, self.cursor_pos, &mut self.state, ctx, *pos);
            }
            InputEvent::Click { button: MouseButton::Left, pos: _ } => {
                if let Some(action_id) = &self.hotbar.selected_action {

                    let action = ctx.resources.actions.get(action_id);
                    let _ = self.action_runner.try_use(
                        action_id,
                        &action,
                        PLAYER_IDX,
                        self.cursor_pos,
                        &mut self.state,
                        &mut self.world,
                        &mut self.game_log,
                        ctx
                    );
                } else {
                    if let Some(path) = &mut self.player_pathing.get_preview() {
                        self.player_pathing.start_running(path.clone());
                    }
                }
            }
            _ => (),
        }
        if self.turn_mode == TurnMode::RealTime {
            self.state.player_mut().ap.fill();
        }
        return ControlFlow::Continue(())
    }

    fn event(&mut self, evt: &BusEvent, ctx: &mut GameContext) -> ControlFlow<()> {
        match evt {
            BusEvent::ShowInspectDialog(data) => {
                self.inspect_dialog.show(InspectDialog::new(data.clone()), &self.world, ctx);
                return ControlFlow::Break(());
            },
            BusEvent::ShowChatDialog(data) => {

                let pending_quest = self.world.codex.quests()
                    .filter(|quest| quest.quest_giver == data.actor.creature_id.unwrap() && quest.status == QuestStatus::RewardPending)
                    .next();

                if let Some(pending_quest) = pending_quest {
                    self.quest_complete_dialog.show(QuestCompleteDialog::new(pending_quest.clone()), &self.world, ctx);
                } else {
                    self.chat_dialog.show(ChatDialog::new(data.clone()), &self.world, ctx);
                }
                return ControlFlow::Break(());
            },
            BusEvent::CreatureKilled(creature_id) => {
                for site_id in self.world.sites.iter_ids::<SiteId>() {

                    let site = self.world.sites.get_mut(&site_id);
                    let creature_lives_here = site.creatures.contains(creature_id);
                    drop(site);
                    if creature_lives_here { 
                        // TODO: Item
                        // TODO: Maybe not player?
                        self.world.creature_kill_creature(*creature_id, site_id, self.state.player().creature_id.unwrap(), None, site_id);
                        break;
                    }
                }

                for quest in self.world.codex.quests_mut() {
                    if let QuestStatus::Complete = quest.status {
                        continue;
                    }
                    let completed = match &quest.objective {
                        QuestObjective::KillVarningr(kill_id) => kill_id == creature_id,
                        QuestObjective::KillBandits(site_id) | QuestObjective::KillWolves(site_id) => {
                            let site = self.world.sites.get(site_id);
                            site.creatures.len() == 0
                        }
                    };
                    if completed {
                        quest.status = QuestStatus::RewardPending;
                    }
                }
                return ControlFlow::Continue(());
            },
            BusEvent::AddItemToPlayer(item) => {
                let _ = self.state.player_mut().inventory.add(item.clone());
                return ControlFlow::Break(());
            },
            BusEvent::PlayerDied => {
                self.death_dialog.show(DeathDialog::new(), &(), ctx);
                return ControlFlow::Break(());
            },
            BusEvent::ConsumeInventoryItem(item) => {
                let actor = self.state.player_mut();
                let item = actor.inventory.take(*item).ok_or("Consumed missing item").unwrap();
                let resources = resources();
                let blueprint = resources.item_blueprints.get(&item.blueprint_id);
                let consumable = blueprint.consumable.as_ref().ok_or("Consumed non-consumable item").unwrap();
                
                for affliction in consumable.effects.iter() {
                    actor.add_affliction(&affliction);
                }
                return ControlFlow::Break(());
            },
            BusEvent::SimulateTime(time) => {
                self.sleep_coroutine = Some((0., *time, false));
                return ControlFlow::Break(());

            },
            _ => ControlFlow::Continue(()),
        }
    }

}