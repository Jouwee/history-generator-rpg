use std::{cell::RefCell, fmt::Display};

use action::{ActionEnum, ActionMap};
use actor::{Actor, ActorType};
use chunk::Chunk;
use codex::{codex_dialog::CodexDialog, knowledge_codex::KnowledgeCodex};
use interact::interact_dialog::InteractDialog;
use inventory::character_dialog::CharacterDialog;
use piston::{Button as Btn, ButtonArgs, ButtonState, Key};

use crate::{engine::{geometry::Coord2, gui::{button::{Button, ButtonEvent}, label::Label, Anchor, GUINode, Position}, render::RenderContext, scene::{Scene, Update}, Color}, world::world::World};

pub mod action;
pub mod actor;
pub mod chunk;
pub mod codex;
pub mod interact;
pub mod inventory;
pub mod log;

pub trait Renderable {
    fn render(&self, ctx: &mut RenderContext);
}

pub struct InputEvent {
    pub mouse_pos: [f64; 2],
    pub button_args: ButtonArgs,
}

pub struct GameSceneState {
    pub world: World,
    pub player: Actor,
    pub codex: KnowledgeCodex,
    pub chunk: Chunk,
    turn_controller: TurnController,
    log: RefCell<Vec<(String, Color)>>,
    actions: ActionMap,
    label: Label,
    button_attack: Button,
    button_talk: Button,
    button_pickup: Button,
    button_codex: Button,
    button_inventory: Button,
    interact_dialog: InteractDialog,
    codex_dialog: CodexDialog,
    inventory_dialog: CharacterDialog,
    selected_targeted_action: Option<ActionEnum>
}

impl GameSceneState {
    pub fn new(world: World, player: Actor, codex: KnowledgeCodex, chunk: Chunk) -> GameSceneState {
        let mut state = GameSceneState {
            world,
            player,
            codex,
            chunk,
            turn_controller: TurnController::new(),
            log: RefCell::new(Vec::new()),
            actions: ActionMap::default(),
            label: Label::new("Stats", Position::Anchored(Anchor::TopLeft, 10.0, 16.0)),
            button_attack: Button::new("atk", Position::Anchored(Anchor::TopLeft, 10.0, 32.0)),
            button_talk: Button::new("tlk", Position::Anchored(Anchor::TopLeft, 36.0, 32.0)),            
            button_pickup: Button::new("pck", Position::Anchored(Anchor::TopLeft, 62.0, 32.0)),            
            button_inventory: Button::new("Character", Position::Anchored(Anchor::TopLeft, 128.0, 32.0)),       
            button_codex: Button::new("Codex", Position::Anchored(Anchor::TopLeft, 228.0, 32.0)),       
            interact_dialog: InteractDialog::new(),
            codex_dialog: CodexDialog::new(),
            inventory_dialog: CharacterDialog::new(),
            selected_targeted_action: None
        };
        state.turn_controller.roll_initiative(state.chunk.npcs.len());
        return state
    }

    pub fn remove_npc(&mut self, i: usize) {
        let id;
        {
            let npc = self.chunk.npcs.get(i).unwrap();
            id = npc.person_id;
            for (_i, item, _equipped) in npc.inventory.iter() {
                self.chunk.items_on_ground.push((npc.xy, item.clone(), item.make_texture(&self.world)));
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

}

impl Scene for GameSceneState {
    fn render(&mut self, ctx: &mut RenderContext) {
        self.chunk.render(ctx);
        self.player.render(ctx);
        for npc in self.chunk.npcs.iter() {
            npc.render(ctx);
        }
        ctx.text("space - end turn", 10, [10.0, 1000.0], Color::from_hex("ffffff"));
        let mut y = 1000.0 - self.log.borrow().len() as f64 * 16.;
        for (line, color) in self.log.borrow().iter() {
            ctx.text(line, 10, [1024.0, y], *color);
            y = y + 16.;
        }

        self.label.render(ctx);
        self.button_attack.render(ctx);
        self.button_talk.render(ctx);
        self.button_pickup.render(ctx);
        self.button_codex.render(ctx);
        self.button_inventory.render(ctx);
        self.interact_dialog.render(ctx);
        self.codex_dialog.render(ctx);
        self.inventory_dialog.render(ctx);        
    }

    fn update(&mut self, _update: &Update) {
        self.label.update();
        self.button_attack.update();
        self.button_talk.update();
        self.button_pickup.update();
        self.button_codex.update();
        self.button_inventory.update();
        self.interact_dialog.update();
        self.codex_dialog.update();
        self.inventory_dialog.update();

        for npc in self.chunk.npcs.iter_mut() {
            npc.update();
        }

        if self.turn_controller.is_player_turn() {
            self.label.text(format!("Player turn | HP: {}/{} | AP: {}/{} | Level: {} | XP: {}", self.player.hp.health_points, self.player.hp.max_health_points, self.player.ap.action_points, self.player.ap.max_action_points, self.player.level, self.player.xp));
        } else {
            self.label.text(format!("Enemy turn {}", self.turn_controller.npc_idx()));
        }
        if self.turn_controller.is_player_turn() {
            return
        }
        let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
        // TODO: AI
        if let ActorType::Hostile = npc.actor_type {
            if npc.xy.dist_squared(&self.player.xy) < 3. {
                if let Ok(log) = self.actions.try_use_on_target(ActionEnum::Attack, npc, &mut self.player) {
                    if let Some(log) = log {
                        self.log(log.string, log.color);
                    }
                }
            } else if npc.ap.can_use(20) {
                if npc.xy.x < self.player.xy.x {
                     if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveRight, npc) {
                        return
                     }
                }
                if npc.xy.x > self.player.xy.x {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveLeft, npc) {
                        return
                    }
                }
                if npc.xy.y < self.player.xy.y {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveDown, npc) {
                        return
                    }
                }
                if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveUp, npc) {
                    return
                }
            }
        }
        let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
        npc.ap.fill();
        self.turn_controller.next_turn();
    }

    fn input(&mut self, evt: &InputEvent) {
        self.interact_dialog.input_state(evt, &self.world, &mut self.codex);
        self.codex_dialog.input_state(evt, &self.world, &mut self.codex);
        self.inventory_dialog.input_state(evt, &mut self.player, &self.world);
        if let ButtonEvent::Click = self.button_attack.event(evt) {
            self.selected_targeted_action = Some(ActionEnum::Attack);
            return;
        }
        if let ButtonEvent::Click = self.button_pickup.event(evt) {
            self.selected_targeted_action = Some(ActionEnum::PickUp);
            return;
        }
        if let ButtonEvent::Click = self.button_talk.event(evt) {
            self.selected_targeted_action = Some(ActionEnum::Talk);
            return;
        }

        if let ButtonEvent::Click = self.button_codex.event(evt) {
            self.codex_dialog.start_dialog();
            return;
        }

        if let ButtonEvent::Click = self.button_inventory.event(evt) {
            self.inventory_dialog.start_dialog(&self.player, &self.world);
            return;
        }

        if !self.turn_controller.is_player_turn() {
            return
        }
        if evt.button_args.state == ButtonState::Press {
            match evt.button_args.button {
                Btn::Keyboard(Key::Space) => {
                    self.turn_controller.next_turn();
                    self.player.ap.fill();
                },
                Btn::Keyboard(Key::Up) => {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveUp, &mut self.player) {
                        return
                    }
                },
                Btn::Keyboard(Key::Down) => {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveDown, &mut self.player) {
                        return
                    }
                },
                Btn::Keyboard(Key::Left) => {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveLeft, &mut self.player) {
                        return
                    }
                },
                Btn::Keyboard(Key::Right) => {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveRight, &mut self.player) {
                        return
                    }
                },
                Btn::Mouse(_any) => {
                    if let Some(action) = &self.selected_targeted_action {
                        let tile_pos = Coord2::xy(evt.mouse_pos[0] as i32 / 16, evt.mouse_pos[1] as i32 / 16);
                        if tile_pos.dist_squared(&self.player.xy) < 3. {
                            let target = self.chunk.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == tile_pos);
                            if let Some((i, target)) = target {

                                match action {
                                    ActionEnum::Attack => {
                                        if let Ok(log) = self.actions.try_use_on_target(ActionEnum::Attack, &mut self.player, target) {
                                            if target.hp.health_points == 0. {
                                                self.player.add_xp(100);
                                                self.log(format!("NPC is dead!"), Color::from_hex("b55945"));
                                                self.remove_npc(i);
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
                                        if let Ok(_) = self.actions.try_use_on_target(ActionEnum::Talk, &mut self.player, target) {
                                            self.interact_dialog.start_dialog(&self.world, target.person_id.unwrap());
                                            return
                                        }
                                    }
                                    _ => ()
                                }

                            }
                        }
                        if tile_pos.dist_squared(&self.player.xy) < 3. {
                            let target = self.chunk.items_on_ground.iter_mut().enumerate().find(|(_i, (xy, _item, _tex))| *xy == tile_pos);
                            if let Some((i, (_xy, item, _texture))) = target {

                                match action {
                                    ActionEnum::PickUp => {
                                        if let Ok(log) = self.actions.try_use_on_item(ActionEnum::PickUp, &mut self.player, item) {
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