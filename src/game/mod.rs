use std::{cell::RefCell, fmt::Display};

use action::{ActionEnum, ActionMap};
use actor::Player;
use chunk::Chunk;
use piston::{Button, ButtonArgs, ButtonState, Key};

use crate::engine::{geometry::Coord2, render::RenderContext, Color};

pub mod action;
pub mod actor;
pub mod chunk;
pub mod log;

pub trait Renderable {
    fn render(&self, ctx: &mut RenderContext);
}

pub trait Scene {
    fn render(&self, ctx: RenderContext);
    fn update(&mut self);
    fn input(&mut self, evt: &InputEvent);
}

pub struct InputEvent {
    pub mouse_pos: [f64; 2],
    pub button_args: ButtonArgs,
}

pub struct GameSceneState {
    pub player: Player,
    pub chunk: Chunk,
    turn_controller: TurnController,
    log: RefCell<Vec<(String, Color)>>,
    actions: ActionMap
}

impl GameSceneState {
    pub fn new(chunk: Chunk) -> GameSceneState {
        let mut state = GameSceneState {
            player: Player::new(Coord2::xy(32, 32)),
            chunk,
            turn_controller: TurnController::new(),
            log: RefCell::new(Vec::new()),
            actions: ActionMap::default()
        };
        state.turn_controller.roll_initiative(state.chunk.npcs.len());
        return state
    }

    pub fn remove_npc(&mut self, i: usize) {
        let id;
        {
            let npc = self.chunk.npcs.get(i).unwrap();
            id = npc.person_id;
        }
        self.chunk.npcs.remove(i);
        self.chunk.killed_people.push(id);
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
    fn render(&self, mut ctx: RenderContext) {
        self.chunk.render(&mut ctx);
        self.player.render(&mut ctx);
        for npc in self.chunk.npcs.iter() {
            npc.render(&mut ctx);
        }
        if self.turn_controller.is_player_turn() {
            let txt = format!("Player turn | HP: {}/{} | AP: {}/{}", self.player.hp.health_points, self.player.hp.max_health_points, self.player.ap.action_points, self.player.ap.max_action_points);
            ctx.text(txt.as_str(), 10, [10.0, 10.0], Color::from_hex("ffffff"));
        } else {
            let txt = format!("Enemy turn {}", self.turn_controller.npc_idx());
            ctx.text(txt.as_str(), 10, [10.0, 10.0], Color::from_hex("ffffff"));
        }
        ctx.text("a - attack     t - talk    space - end turn", 10, [10.0, 1000.0], Color::from_hex("ffffff"));
        let mut y = 1000.0 - self.log.borrow().len() as f64 * 16.;
        for (line, color) in self.log.borrow().iter() {
            ctx.text(line, 10, [1024.0, y], *color);
            y = y + 16.;
        }
        
    }

    fn update(&mut self) {
        if self.turn_controller.is_player_turn() {
            return
        }
        println!("AI Turn: {}", self.turn_controller.npc_idx());
        let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
        // TODO: AI
        if npc.hostile {
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
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveUp, npc) {
                        return
                    }
                }
                if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveDown, npc) {
                    return
                }
            }
        }
        let npc = self.chunk.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
        npc.ap.fill();
        self.turn_controller.next_turn();
    }

    fn input(&mut self, evt: &InputEvent) {
        println!("Player turn? {}", self.turn_controller.is_player_turn());
        if !self.turn_controller.is_player_turn() {
            return
        }

        if evt.button_args.state == ButtonState::Press {
            match evt.button_args.button {
                Button::Keyboard(Key::Space) => {
                    println!("End turn");
                    self.turn_controller.next_turn();
                    self.player.ap.fill();
                },
                Button::Keyboard(Key::Up) => {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveUp, &mut self.player) {
                        return
                    }
                },
                Button::Keyboard(Key::Down) => {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveDown, &mut self.player) {
                        return
                    }
                },
                Button::Keyboard(Key::Left) => {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveLeft, &mut self.player) {
                        return
                    }
                },
                Button::Keyboard(Key::Right) => {
                    if let Ok(_) = self.actions.try_use_on_self(ActionEnum::MoveRight, &mut self.player) {
                        return
                    }
                },
                Button::Keyboard(Key::A) => {
                    let tile_pos = Coord2::xy(evt.mouse_pos[0] as i32 / 16, evt.mouse_pos[1] as i32 / 16);
                    if self.player.ap.can_use(40) && tile_pos.dist_squared(&self.player.xy) < 3. {
                        let target = self.chunk.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == tile_pos);
                        if let Some((i, target)) = target {

                            if let Ok(_) = self.actions.try_use_on_target(ActionEnum::Attack, &mut self.player, target) {
                                target.hostile = true;
                                // TODO: ?????
                                // if let Some(log) = log {
                                    // self.log(log.string, log.color);
                                // }
                                // self.log(format!("You attack NPC for ???"), Color::from_hex("eb9661"));
                                if target.hp.health_points == 0. {
                                    self.log(format!("NPC is dead!"), Color::from_hex("b55945"));
                                    self.remove_npc(i);
                                }
                            }
                        } else {
                            println!("No target {:?}", tile_pos);
                        }
                    }
                },
                Button::Keyboard(Key::T) => {
                    let tile_pos = Coord2::xy(evt.mouse_pos[0] as i32 / 16, evt.mouse_pos[1] as i32 / 16);
                    if tile_pos.dist_squared(&self.player.xy) < 3. {
                        let target = self.chunk.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == tile_pos);
                        if let Some((_, target)) = target {
                            if !target.hostile {
                                let txt = format!("Hello! I am {:?}", target.person.name());
                                self.log(txt, Color::from_hex("cae6d9"));
                            }
                        }
                    }
                },
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