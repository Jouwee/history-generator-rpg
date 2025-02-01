use std::{cell::RefCell, collections::HashMap, fmt::Display};

use action::{ActionEnum, ActionMap};
use actor::ActorType;
use chunk::Chunk;
use codex::{codex_dialog::CodexDialog, knowledge_codex::KnowledgeCodex};
use hotbar::{Hotbar, HotbarState, NodeWithState};
use interact::interact_dialog::InteractDialog;
use inventory::character_dialog::CharacterDialog;
use piston::{Button as Btn, ButtonArgs, ButtonState, Key};

use crate::{engine::{animation::Animation, audio::TrackMood, geometry::Coord2, gui::{button::{Button, ButtonEvent}, Anchor, GUINode, Position}, render::RenderContext, scene::{Scene, Update}, Color}, world::world::World, GameContext};

pub mod action;
pub mod actor;
pub mod chunk;
pub mod codex;
pub mod hotbar;
pub mod interact;
pub mod inventory;
pub mod log;

pub trait Renderable {
    fn render(&self, ctx: &mut RenderContext, game_ctx: &GameContext);
}

pub struct InputEvent {
    pub mouse_pos_cam: [f64; 2],
    pub mouse_pos_gui: [f64; 2],
    pub button_args: ButtonArgs,
}

pub struct GameSceneState {
    pub world: World,
    pub codex: KnowledgeCodex,
    pub world_pos: Coord2,
    pub chunk: Chunk,
    turn_controller: TurnController,
    log: RefCell<Vec<(String, Color)>>,
    actions: ActionMap,
    button_codex: Button,
    button_inventory: Button,
    hotbar: Hotbar,
    interact_dialog: InteractDialog,
    codex_dialog: CodexDialog,
    inventory_dialog: CharacterDialog,
    cursor_pos: Coord2
}

impl GameSceneState {
    pub fn new(world: World, world_pos: Coord2, codex: KnowledgeCodex, chunk: Chunk) -> GameSceneState {
        let mut state = GameSceneState {
            world,
            codex,
            chunk,
            world_pos,
            turn_controller: TurnController::new(),
            log: RefCell::new(Vec::new()),
            actions: ActionMap::default(),
            hotbar: Hotbar::new(),
            button_inventory: Button::new("Character", Position::Anchored(Anchor::BottomLeft, 10.0, 32.0)),       
            button_codex: Button::new("Codex", Position::Anchored(Anchor::BottomLeft, 64.0, 32.0)),       
            interact_dialog: InteractDialog::new(),
            codex_dialog: CodexDialog::new(),
            inventory_dialog: CharacterDialog::new(),
            cursor_pos: Coord2::xy(0, 0)
        };
        state.save_creature_appearances();
        state.turn_controller.roll_initiative(state.chunk.npcs.len());
        return state
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

    pub fn log(&self, text: impl Display, color: Color) {
        let mut log = self.log.borrow_mut();
        log.push((text.to_string(), color));
        if log.len() > 50 {
            log.pop();
        }
    }

    fn build_walk_anim() -> Animation {
        Animation::new()
            .translate(0.08, [0., -6.], crate::engine::animation::Smoothing::EaseInOut)
            .translate(0.08, [0., 0.], crate::engine::animation::Smoothing::EaseInOut)

    }

    fn build_hurt_anim(direction: Coord2) -> Animation {
        let direction = direction.to_vec2().normalize(12.);
        Animation::new()
            .translate(0.02, [direction.x as f64, direction.y as f64], crate::engine::animation::Smoothing::EaseInOut)
            .translate(0.2, [0., 0.], crate::engine::animation::Smoothing::EaseInOut)
    }

    fn build_attack_anim(direction: Coord2) -> Animation {
        let direction = direction.to_vec2().normalize(24.);
        Animation::new()
            .translate(0.08, [direction.x as f64, direction.y as f64], crate::engine::animation::Smoothing::EaseInOut)
            .translate(0.08, [0., 0.], crate::engine::animation::Smoothing::EaseInOut)
    }

}

impl Scene for GameSceneState {
    fn init(&mut self, ctx: &mut GameContext) {
        if self.chunk.npcs.iter().find(|actor| actor.actor_type == ActorType::Hostile).is_some() {
            ctx.audio.switch_music(TrackMood::Battle);
        } else {
            ctx.audio.switch_music(TrackMood::Regular);
        }
    }

    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &GameContext) {
        ctx.pixel_art(2);
        let center = self.chunk.player.xy;
        ctx.push();
        ctx.center_camera_on([center.x as f64 * 24., center.y as f64 * 24.]);
        self.chunk.render(ctx, &game_ctx);

        if let Some(_) = self.hotbar.selected_action {
            ctx.image("cursor.png", [self.cursor_pos.x as f64 * 24., self.cursor_pos.y as f64 * 24.]);
        }

        let _ = ctx.try_pop();
        let mut y = 1000.0 - self.log.borrow().len() as f64 * 16.;
        for (line, color) in self.log.borrow().iter() {
            ctx.text(line, 10, [1024.0, y], *color);
            y = y + 16.;
        }
        self.hotbar.render(HotbarState::new(&self.chunk.player), ctx);
        self.button_codex.render(ctx);
        self.button_inventory.render(ctx);
        self.interact_dialog.render(ctx);
        self.codex_dialog.render(ctx);
        self.inventory_dialog.render(ctx);        
    }

    fn update(&mut self, update: &Update, ctx: &mut GameContext) {
        self.hotbar.update(HotbarState::new(&self.chunk.player), update);
        self.button_codex.update();
        self.button_inventory.update();
        self.interact_dialog.update();
        self.codex_dialog.update();
        self.inventory_dialog.update();

        self.cursor_pos = Coord2::xy((update.mouse_pos_cam[0] / 24.) as i32, (update.mouse_pos_cam[1] / 24.) as i32);

        let mut hostile = false;
        for npc in self.chunk.npcs.iter_mut() {
            npc.update(update.delta_time);
            hostile = hostile || npc.actor_type == ActorType::Hostile;
        }
        self.chunk.player.update(update.delta_time);
        if hostile {
            ctx.audio.switch_music(TrackMood::Battle);
        } else {
            ctx.audio.switch_music(TrackMood::Regular);
        }

        if self.turn_controller.is_player_turn() {
            return
        }
        let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
        // TODO: AI
        if let ActorType::Hostile = npc.actor_type {
            if npc.xy.dist_squared(&self.chunk.player.xy) < 3. {
                if let Ok(log) = self.actions.try_use_on_target(ActionEnum::Attack, npc, &mut self.chunk.player) {
                    if let Some(fx) = ActionEnum::Attack.sound_effect() {
                        ctx.audio.play_once(fx);
                    }
                    let dir = self.chunk.player.xy - npc.xy;
                    npc.animation.play(&Self::build_attack_anim(dir));
                    self.chunk.player.animation.play(&Self::build_hurt_anim(dir));
                    if let Some(log) = log {
                        self.log(log.string, log.color);
                    }
                }
            } else if npc.ap.can_use(20) {
                let xy = npc.xy.clone();
                if npc.xy.x < self.chunk.player.xy.x {
                    if let Ok(_) = self.actions.try_use_on_tile(ActionEnum::MoveRight, npc, &mut self.chunk.map, &xy) {
                        let xy = npc.xy.clone();
                        npc.animation.play(&Self::build_walk_anim());
                        if let Some(sound) = self.chunk.get_step_sound(xy) {
                            // TODO: Use actual camera
                            ctx.audio.play_positional(sound, xy.to_vec2(), self.chunk.player.xy.to_vec2());
                        }
                        return
                    }
                }
                if npc.xy.x > self.chunk.player.xy.x {
                    if let Ok(_) = self.actions.try_use_on_tile(ActionEnum::MoveLeft, npc, &mut self.chunk.map, &xy) {
                        let xy = npc.xy.clone();
                        npc.animation.play(&Self::build_walk_anim());
                        if let Some(sound) = self.chunk.get_step_sound(xy) {
                            // TODO: Use actual camera
                            ctx.audio.play_positional(sound, xy.to_vec2(), self.chunk.player.xy.to_vec2());
                        }
                        return
                    }
                }
                if npc.xy.y < self.chunk.player.xy.y {
                    if let Ok(_) = self.actions.try_use_on_tile(ActionEnum::MoveDown, npc, &mut self.chunk.map, &xy) {
                        let xy = npc.xy.clone();
                        npc.animation.play(&Self::build_walk_anim());
                        if let Some(sound) = self.chunk.get_step_sound(xy) {
                            // TODO: Use actual camera
                            ctx.audio.play_positional(sound, xy.to_vec2(), self.chunk.player.xy.to_vec2());
                        }
                        return
                    }
                }
                if let Ok(_) = self.actions.try_use_on_tile(ActionEnum::MoveUp, npc, &mut self.chunk.map, &xy) {
                    let xy = npc.xy.clone();
                    npc.animation.play(&Self::build_walk_anim());
                    if let Some(sound) = self.chunk.get_step_sound(xy) {
                        // TODO: Use actual camera
                        ctx.audio.play_positional(sound, xy.to_vec2(), self.chunk.player.xy.to_vec2());
                    }
                    return
                }
            }
        }
        let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
        npc.ap.fill();
        self.turn_controller.next_turn();
    }

    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) {
        self.hotbar.input(HotbarState::new(&self.chunk.player), evt);
        self.interact_dialog.input_state(evt, &self.world, &ctx.resources, &mut self.codex);
        self.codex_dialog.input_state(evt, &self.world, &ctx.resources, &mut self.codex);
        self.inventory_dialog.input_state(evt, &mut self.chunk.player, &ctx.resources);
        if let ButtonEvent::Click = self.button_codex.event(evt) {
            self.codex_dialog.start_dialog();
            return;
        }

        if let ButtonEvent::Click = self.button_inventory.event(evt) {
            self.inventory_dialog.start_dialog(&self.chunk.player, &ctx.resources);
            return;
        }

        if !self.turn_controller.is_player_turn() {
            return
        }
        if evt.button_args.state == ButtonState::Press {
            let xy = self.chunk.player.xy.clone();
            match evt.button_args.button {
                Btn::Keyboard(Key::Space) => {
                    self.turn_controller.next_turn();
                    self.chunk.player.ap.fill();
                },
                Btn::Keyboard(Key::Up) => {
                    if let Ok(_) = self.actions.try_use_on_tile(ActionEnum::MoveUp, &mut self.chunk.player, &mut self.chunk.map, &xy) {
                        let xy = self.chunk.player.xy.clone();
                        if let Some(sound) = self.chunk.get_step_sound(xy) {
                            ctx.audio.play_once(sound);
                        }
                        self.chunk.player.animation.play(&Self::build_walk_anim());
                        return
                    }
                },
                Btn::Keyboard(Key::Down) => {
                    if let Ok(_) = self.actions.try_use_on_tile(ActionEnum::MoveDown, &mut self.chunk.player, &mut self.chunk.map, &xy) {
                        let xy = self.chunk.player.xy.clone();
                        if let Some(sound) = self.chunk.get_step_sound(xy) {
                            ctx.audio.play_once(sound);
                        }
                        self.chunk.player.animation.play(&Self::build_walk_anim());
                        return
                    }
                },
                Btn::Keyboard(Key::Left) => {
                    if let Ok(_) = self.actions.try_use_on_tile(ActionEnum::MoveLeft, &mut self.chunk.player, &mut self.chunk.map, &xy) {
                        let xy = self.chunk.player.xy.clone();
                        if let Some(sound) = self.chunk.get_step_sound(xy) {
                            ctx.audio.play_once(sound);
                        }
                        self.chunk.player.animation.play(&Self::build_walk_anim());
                        return
                    }
                },
                Btn::Keyboard(Key::Right) => {
                    if let Ok(_) = self.actions.try_use_on_tile(ActionEnum::MoveRight, &mut self.chunk.player, &mut self.chunk.map, &xy) {
                        let xy = self.chunk.player.xy.clone();
                        if let Some(sound) = self.chunk.get_step_sound(xy) {
                            ctx.audio.play_once(sound);
                        }
                        self.chunk.player.animation.play(&Self::build_walk_anim());
                        return
                    }
                },
                Btn::Mouse(_any) => {
                    if let Some(action) = &self.hotbar.selected_action {
                        let tile_pos = Coord2::xy(evt.mouse_pos_cam[0] as i32 / 24, evt.mouse_pos_cam[1] as i32 / 24);
                        if tile_pos.dist_squared(&self.chunk.player.xy) < 3. {
                            let target = self.chunk.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == tile_pos);
                            if let Some((i, target)) = target {

                                match action {
                                    ActionEnum::Attack | ActionEnum::UnarmedAttack => {
                                        if let Ok(log) = self.actions.try_use_on_target(ActionEnum::Attack, &mut self.chunk.player, target) {

                                            if let Some(fx) = action.sound_effect() {
                                                ctx.audio.play_once(fx);
                                            }
                                            let dir = target.xy - self.chunk.player.xy;
                                            self.chunk.player.animation.play(&Self::build_attack_anim(dir));
                                            target.animation.play(&&Self::build_hurt_anim(dir));
                                            if target.hp.health_points == 0. {
                                                self.chunk.player.add_xp(100);
                                                self.log(format!("NPC is dead!"), Color::from_hex("b55945"));
                                                self.remove_npc(i, ctx);
                                            } else if let Some(log) = log {
                                                self.log(log.string, log.color);
                                            }
                                            // Turn everyone hostile
                                            for p in self.chunk.npcs.iter_mut() {
                                                p.actor_type = ActorType::Hostile;
                                            }
                                            return
                                        }
                                    }

                                    ActionEnum::Talk => {
                                        if let Ok(_) = self.actions.try_use_on_target(ActionEnum::Talk, &mut self.chunk.player, target) {
                                            self.interact_dialog.start_dialog(&self.world, target.person_id.unwrap());
                                            return
                                        }
                                    }
                                    _ => ()
                                }

                            }
                            let target = self.chunk.items_on_ground.iter_mut().enumerate().find(|(_i, (xy, _item, _tex))| *xy == tile_pos);
                            if let Some((i, (_xy, item, _texture))) = target {

                                match action {
                                    ActionEnum::PickUp => {
                                        if let Ok(log) = self.actions.try_use_on_item(ActionEnum::PickUp, &mut self.chunk.player, item) {
                                            if let Some(log) = log {
                                                self.log(log.string, log.color);
                                            }
                                            self.chunk.items_on_ground.remove(i);
                                            return
                                        }
                                    }
                                    _ => ()
                                }

                            }
                            match action {
                                ActionEnum::Sleep => {
                                    if let Ok(log) = self.actions.try_use_on_tile(ActionEnum::Sleep, &mut self.chunk.player, &mut self.chunk.map, &tile_pos) {
                                        if let Some(log) = log {
                                            self.log(log.string, log.color);
                                        }
                                        return
                                    }
                                }
                                _ => ()
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }

    fn cursor_move(&mut self, _pos: [f64; 2]) {

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