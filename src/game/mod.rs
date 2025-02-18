use std::{cell::RefCell, collections::HashMap, fmt::Display};

use action::{ActionTargetOutput, ActionType, DamageOutput};
use actor::ActorType;
use chunk::Chunk;
use codex::{codex_dialog::CodexDialog, knowledge_codex::KnowledgeCodex};
use effect_layer::EffectLayer;
use hotbar::{Hotbar, HotbarState, NodeWithState};
use interact::interact_dialog::InteractDialog;
use inventory::character_dialog::CharacterDialog;
use piston::{Button as Btn, ButtonArgs, ButtonState, Key};

use crate::{engine::{animation::Animation, audio::TrackMood, geometry::Coord2, gui::{button::{Button, ButtonEvent}, Anchor, GUINode, Position}, render::RenderContext, scene::{Scene, Update}, Color}, world::world::World, GameContext};

pub mod action;
pub mod actor;
pub mod chunk;
pub mod codex;
pub mod effect_layer;
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
    button_codex: Button,
    button_inventory: Button,
    hotbar: Hotbar,
    interact_dialog: InteractDialog,
    codex_dialog: CodexDialog,
    inventory_dialog: CharacterDialog,
    cursor_pos: Coord2,
    effect_layer: EffectLayer,
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
            hotbar: Hotbar::new(),
            button_inventory: Button::new("Character", Position::Anchored(Anchor::BottomLeft, 10.0, 32.0)),       
            button_codex: Button::new("Codex", Position::Anchored(Anchor::BottomLeft, 64.0, 32.0)),       
            interact_dialog: InteractDialog::new(),
            codex_dialog: CodexDialog::new(),
            inventory_dialog: CharacterDialog::new(),
            cursor_pos: Coord2::xy(0, 0),
            effect_layer: EffectLayer::new()
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
        self.hotbar.init(ctx);
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
        // Game
        ctx.center_camera_on([center.x as f64 * 24., center.y as f64 * 24.]);
        self.chunk.render(ctx, &game_ctx);

        if let Some(_) = self.hotbar.selected_action {
            ctx.image("cursor.png", [self.cursor_pos.x as f64 * 24., self.cursor_pos.y as f64 * 24.]);
        }
        // Effects
        self.effect_layer.render(ctx, game_ctx);
        // UI
        let _ = ctx.try_pop();
        let mut y = 1000.0 - self.log.borrow().len() as f64 * 16.;
        for (line, color) in self.log.borrow().iter() {
            ctx.text(line, 10, [1024.0, y], *color);
            y = y + 16.;
        }
        self.hotbar.render(HotbarState::new(&self.chunk.player), ctx, game_ctx);
        self.button_codex.render(ctx);
        self.button_inventory.render(ctx);
        self.interact_dialog.render(ctx);
        self.codex_dialog.render(ctx);
        self.inventory_dialog.render(ctx);        
    }

    fn update(&mut self, update: &Update, ctx: &mut GameContext) {
        self.hotbar.update(HotbarState::new(&self.chunk.player), update, ctx);
        self.button_codex.update();
        self.button_inventory.update();
        self.interact_dialog.update();
        self.codex_dialog.update();
        self.inventory_dialog.update();
        self.effect_layer.update(update, ctx);

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
                let action = match npc.inventory.equipped() {                
                    Some(_) => ctx.resources.actions.find("act:sword:attack"),
                    None => ctx.resources.actions.find("act:punch")
                };
                if npc.ap.can_use(action.ap_cost) {
                    // TODO: Dupped code
                    if let ActionType::Targeted { damage } = &action.action_type {
                        if let Some(damage) = damage {
                            let damage = match &damage {
                                action::DamageType::Fixed(dmg) => dmg,
                                action::DamageType::FromWeapon => {
                                    let item = self.chunk.player.inventory.equipped().expect("Used equipped action with no equipped item");
                                    &item.damage_model()
                                }
                            };

                            let str_mult = self.chunk.player.attributes.strength_attack_damage_mult();
                            let damage_model = damage.multiply(str_mult);
                            let damage = damage_model.resolve(&self.chunk.player.defence);

                            if let Some(fx) = &action.sound_effect {
                                ctx.audio.play_once(fx.clone());
                            }
                            self.effect_layer.add_damage_number(self.chunk.player.xy, damage);
                            let dir = self.chunk.player.xy - npc.xy;
                            npc.animation.play(&Self::build_attack_anim(dir));
                            self.chunk.player.animation.play(&Self::build_hurt_anim(dir));
                        }
                    }
                }
            } else if npc.ap.can_use(20) {
                if npc.xy.x < self.chunk.player.xy.x {
                    let action = ctx.resources.actions.find("act:move_right");  
                    if npc.ap.can_use(action.ap_cost) {
                        if let ActionType::Move { offset } = action.action_type {
                            let xy = npc.xy.clone();
                            let pos = xy + offset;
                            if !self.chunk.map.blocks_movement(pos) {
                                npc.ap.consume(action.ap_cost);
                                npc.xy = npc.xy + offset;
                                npc.animation.play(&Self::build_walk_anim());
                                if let Some(sound) = self.chunk.get_step_sound(xy) {
                                    // TODO: Use actual camera
                                    ctx.audio.play_positional(sound, xy.to_vec2(), self.chunk.player.xy.to_vec2());
                                }
                                return
                            }
                        }
                    }
                }
                if npc.xy.x > self.chunk.player.xy.x {
                    let action = ctx.resources.actions.find("act:move_left");  
                    if npc.ap.can_use(action.ap_cost) {
                        if let ActionType::Move { offset } = action.action_type {
                            let xy = npc.xy.clone();
                            let pos = xy + offset;
                            if !self.chunk.map.blocks_movement(pos) {
                                npc.ap.consume(action.ap_cost);
                                npc.xy = npc.xy + offset;
                                npc.animation.play(&Self::build_walk_anim());
                                if let Some(sound) = self.chunk.get_step_sound(xy) {
                                    // TODO: Use actual camera
                                    ctx.audio.play_positional(sound, xy.to_vec2(), self.chunk.player.xy.to_vec2());
                                }
                                return
                            }
                        }
                    }
                }
                if npc.xy.y < self.chunk.player.xy.y {
                    let action = ctx.resources.actions.find("act:move_down");  
                    if npc.ap.can_use(action.ap_cost) {
                        if let ActionType::Move { offset } = action.action_type {
                            let xy = npc.xy.clone();
                            let pos = xy + offset;
                            if !self.chunk.map.blocks_movement(pos) {
                                npc.ap.consume(action.ap_cost);
                                npc.xy = npc.xy + offset;
                                npc.animation.play(&Self::build_walk_anim());
                                if let Some(sound) = self.chunk.get_step_sound(xy) {
                                    // TODO: Use actual camera
                                    ctx.audio.play_positional(sound, xy.to_vec2(), self.chunk.player.xy.to_vec2());
                                }
                                return
                            }
                        }
                    }
                }
                let action = ctx.resources.actions.find("act:move_up");  
                if npc.ap.can_use(action.ap_cost) {
                    if let ActionType::Move { offset } = action.action_type {
                        let xy = npc.xy.clone();
                        let pos = xy + offset;
                        if !self.chunk.map.blocks_movement(pos) {
                            npc.ap.consume(action.ap_cost);
                            npc.xy = npc.xy + offset;
                            npc.animation.play(&Self::build_walk_anim());
                            if let Some(sound) = self.chunk.get_step_sound(xy) {
                                // TODO: Use actual camera
                                ctx.audio.play_positional(sound, xy.to_vec2(), self.chunk.player.xy.to_vec2());
                            }
                            return
                        }
                    }
                }
            }
        }
        let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
        npc.ap.fill();
        self.turn_controller.next_turn();
    }

    fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) {
        self.hotbar.input(HotbarState::new(&self.chunk.player), evt, ctx);
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
            match evt.button_args.button {
                Btn::Keyboard(Key::Space) => {
                    self.turn_controller.next_turn();
                    self.chunk.player.ap.fill();
                },
                Btn::Keyboard(Key::Up) => {
                    let action = ctx.resources.actions.find("act:move_up");  
                    if self.chunk.player.ap.can_use(action.ap_cost) {
                        if let ActionType::Move { offset } = action.action_type {
                            let xy = self.chunk.player.xy.clone();
                            let pos = xy + offset;
                            if !self.chunk.map.blocks_movement(pos) {
                                self.chunk.player.ap.consume(action.ap_cost);
                                self.chunk.player.xy = self.chunk.player.xy + offset;
                                self.chunk.player.animation.play(&Self::build_walk_anim());
                                if let Some(sound) = self.chunk.get_step_sound(xy) {
                                    ctx.audio.play_once(sound);
                                }
                            }
                        }
                    }
                },
                Btn::Keyboard(Key::Down) => {
                    let action = ctx.resources.actions.find("act:move_down");  
                    if self.chunk.player.ap.can_use(action.ap_cost) {
                        if let ActionType::Move { offset } = action.action_type {
                            let xy = self.chunk.player.xy.clone();
                            let pos = xy + offset;
                            if !self.chunk.map.blocks_movement(pos) {
                                self.chunk.player.ap.consume(action.ap_cost);
                                self.chunk.player.xy = self.chunk.player.xy + offset;
                                self.chunk.player.animation.play(&Self::build_walk_anim());
                                if let Some(sound) = self.chunk.get_step_sound(xy) {
                                    ctx.audio.play_once(sound);
                                }
                            }
                        }
                    }
                },
                Btn::Keyboard(Key::Left) => {
                    let action = ctx.resources.actions.find("act:move_left");  
                    if self.chunk.player.ap.can_use(action.ap_cost) {
                        if let ActionType::Move { offset } = action.action_type {
                            let xy = self.chunk.player.xy.clone();
                            let pos = xy + offset;
                            if !self.chunk.map.blocks_movement(pos) {
                                self.chunk.player.ap.consume(action.ap_cost);
                                self.chunk.player.xy = self.chunk.player.xy + offset;
                                self.chunk.player.animation.play(&Self::build_walk_anim());
                                if let Some(sound) = self.chunk.get_step_sound(xy) {
                                    ctx.audio.play_once(sound);
                                }
                            }
                        }
                    }
                },
                Btn::Keyboard(Key::Right) => {
                    let action = ctx.resources.actions.find("act:move_right");  
                    if self.chunk.player.ap.can_use(action.ap_cost) {
                        if let ActionType::Move { offset } = action.action_type {
                            let xy = self.chunk.player.xy.clone();
                            let pos = xy + offset;
                            if !self.chunk.map.blocks_movement(pos) {
                                self.chunk.player.ap.consume(action.ap_cost);
                                self.chunk.player.xy = self.chunk.player.xy + offset;
                                self.chunk.player.animation.play(&Self::build_walk_anim());
                                if let Some(sound) = self.chunk.get_step_sound(xy) {
                                    ctx.audio.play_once(sound);
                                }
                            }
                        }
                    }
                },
                Btn::Mouse(_any) => {

                    if let Some(action_id) = &self.hotbar.selected_action {
                        let action = ctx.resources.actions.get(action_id);
                        if self.chunk.player.ap.can_use(action.ap_cost) {
                            match &action.action_type {
                                ActionType::Targeted { damage } => {
                                    // TODO: Dupped code
                                    let tile_pos = Coord2::xy(evt.mouse_pos_cam[0] as i32 / 24, evt.mouse_pos_cam[1] as i32 / 24);
                                    if tile_pos.dist_squared(&self.chunk.player.xy) < 3. {
                                        let target = self.chunk.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == tile_pos);
                                        if let Some((i, target)) = target {
                                            self.chunk.player.ap.consume(action.ap_cost);


                                            if let Some(damage) = damage {
                                                
                                                let damage = match &damage {
                                                    action::DamageType::Fixed(dmg) => dmg,
                                                    action::DamageType::FromWeapon => {
                                                        let item = self.chunk.player.inventory.equipped().expect("Used equipped action with no equipped item");
                                                        &item.damage_model()
                                                    }
                                                };

                                                let str_mult = self.chunk.player.attributes.strength_attack_damage_mult();
                                                let damage_model = damage.multiply(str_mult);
                                                let damage = damage_model.resolve(&target.defence);
                                                target.hp.damage(damage);
                                                let result = ActionTargetOutput {
                                                    damage: Some(DamageOutput { damage })
                                                };

                                                if let Some(fx) = &action.sound_effect {
                                                    ctx.audio.play_once(fx.clone());
                                                }
                                                if let Some(damage) = result.damage  {
                                                    self.effect_layer.add_damage_number(target.xy, damage.damage);
                                                }
                                                let dir = target.xy - self.chunk.player.xy;
                                                self.chunk.player.animation.play(&Self::build_attack_anim(dir));
                                                target.animation.play(&&Self::build_hurt_anim(dir));
                                                if target.hp.health_points == 0. {
                                                    self.chunk.player.add_xp(100);
                                                    self.log(format!("NPC is dead!"), Color::from_hex("b55945"));
                                                    self.remove_npc(i, ctx);
                                                }
                                                // Turn everyone hostile
                                                for p in self.chunk.npcs.iter_mut() {
                                                    p.actor_type = ActorType::Hostile;
                                                }
                                                return


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