use std::{cell::RefCell, fmt::Display};

use piston::{Button, ButtonArgs, ButtonState, Key};

use crate::engine::{render::RenderContext, Color, Point2D};

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
    pub npcs: Vec<NPC>,
    turn_controller: TurnController,
    log: RefCell<Vec<(String, Color)>>
}

impl GameSceneState {
    pub fn new() -> GameSceneState {
        let mut state = GameSceneState {
            player: Player::new(Point2D(32, 32)),
            npcs: vec!(
                NPC::new(Point2D(25, 25)),
                NPC::new(Point2D(45, 45)),
                NPC::new(Point2D(45, 25)),
                NPC::new(Point2D(25, 45)),
            ),
            turn_controller: TurnController::new(),
            log: RefCell::new(Vec::new())
        };
        state.turn_controller.roll_initiative(state.npcs.len());
        return state
    }

    pub fn remove_npc(&mut self, i: usize) {
        self.npcs.remove(i);
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
        self.player.render(&mut ctx);
        for npc in self.npcs.iter() {
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
        let npc = self.npcs.get_mut(self.turn_controller.npc_idx()).unwrap();
        // TODO: AI
        if npc.hostile {
            if npc.xy.dist_squared(&self.player.xy) < 3. {
                if npc.ap.can_use(40) {
                    println!("Got attacked!");
                    npc.ap.consume(40);
                    let damage = npc.damage.resolve(&self.player.defence);
                    self.log(format!("NPC attacks you for {damage}"), Color::from_hex("eb9661"));
                    self.player.hp.damage(damage);
                    return
                }
            } else if npc.ap.can_use(20) {
                if npc.xy.0 < self.player.xy.0 {
                    npc.xy.0 += 1;
                } else if npc.xy.0 > self.player.xy.0 {
                    npc.xy.0 -= 1;
                } else if npc.xy.1 < self.player.xy.1 {
                    npc.xy.1 += 1;
                } else {
                    npc.xy.1 -= 1;
                }
                npc.ap.consume(20);
                return
            }
        }
        npc.ap.fill();
        self.turn_controller.next_turn();
    }

    fn input(&mut self, evt: &InputEvent) {
        println!("Player turn? {}", self.turn_controller.is_player_turn());
        if !self.turn_controller.is_player_turn() {
            return
        }

        let movement_ap_cost = 20;

        if evt.button_args.state == ButtonState::Press {
            match evt.button_args.button {
                Button::Keyboard(Key::Space) => {
                    println!("End turn");
                    self.turn_controller.next_turn();
                    self.player.ap.fill();
                },
                Button::Keyboard(Key::Up) => {
                    if self.player.ap.can_use(movement_ap_cost) && self.player.xy.1 > 0 {
                        self.player.xy.1 -= 1;
                        self.player.ap.consume(movement_ap_cost);
                    }
                },
                Button::Keyboard(Key::Down) => {
                    if self.player.ap.can_use(movement_ap_cost) && self.player.xy.1 < 63 {
                        self.player.xy.1 += 1;
                        self.player.ap.consume(movement_ap_cost);
                    }
                },
                Button::Keyboard(Key::Left) => {
                    if self.player.ap.can_use(movement_ap_cost) && self.player.xy.0 > 0 {
                        self.player.xy.0 -= 1;
                        self.player.ap.consume(movement_ap_cost);
                    }
                },
                Button::Keyboard(Key::Right) => {
                    if self.player.ap.can_use(movement_ap_cost) && self.player.xy.0 < 63 {
                        self.player.xy.0 += 1;
                        self.player.ap.consume(movement_ap_cost);
                    }
                },
                Button::Keyboard(Key::A) => {
                    let tile_pos = Point2D(evt.mouse_pos[0] as usize / 16, evt.mouse_pos[1] as usize / 16);
                    if self.player.ap.can_use(40) && tile_pos.dist_squared(&self.player.xy) < 3. {
                        let target = self.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == tile_pos);
                        if let Some((i, target)) = target {
                            println!("Attack! {:?}", tile_pos);
                            self.player.ap.consume(40);
                            target.hostile = true;
                            let damage = self.player.damage.resolve(&target.defence);
                            target.hp.damage(damage);
                            let hp = target.hp.health_points;
                            self.log(format!("You attack NPC for {damage}"), Color::from_hex("eb9661"));
                            if hp == 0. {
                                self.log(format!("NPC is dead!"), Color::from_hex("b55945"));
                                self.remove_npc(i);
                            }
                        } else {
                            println!("No target {:?}", tile_pos);
                        }
                    }
                },
                Button::Keyboard(Key::T) => {
                    let tile_pos = Point2D(evt.mouse_pos[0] as usize / 16, evt.mouse_pos[1] as usize / 16);
                    if tile_pos.dist_squared(&self.player.xy) < 3. {
                        let target = self.npcs.iter_mut().enumerate().find(|(_, npc)| npc.xy == tile_pos);
                        if let Some((i, target)) = target {
                            if !target.hostile {
                                self.log("Hello!", Color::from_hex("cae6d9"));
                            }
                        }
                    }
                },
                _ => (),
            }
        }
    }
}

pub struct Player {
    pub xy: Point2D,
    pub ap: ActionPointsComponent,
    pub hp: HealthPointsComponent,
    pub damage: DamageComponent,
    pub defence: DefenceComponent
}

impl Player {
    pub fn new(xy: Point2D) -> Player {
        Player {
            xy,
            ap: ActionPointsComponent::new(100),
            hp: HealthPointsComponent::new(100.),
            damage: DamageComponent { slashing: 10.0 },
            defence: DefenceComponent { slashing: 3.0 }
        }
    }
}

impl Renderable for Player {
    fn render(&self, ctx: &mut RenderContext) {
        ctx.rectangle_fill([self.xy.0 as f64 * 16.0, self.xy.1 as f64 * 16.0, 16.0, 16.0], Color::from_hex("00ffff"));
    }
}

pub struct NPC {
    pub xy: Point2D,
    pub ap: ActionPointsComponent,
    pub hp: HealthPointsComponent,
    pub damage: DamageComponent,
    pub defence: DefenceComponent,
    pub hostile: bool
}

impl NPC {
    pub fn new(xy: Point2D) -> NPC {
        NPC {
            xy,
            ap: ActionPointsComponent::new(80),
            hp: HealthPointsComponent::new(50.),
            damage: DamageComponent { slashing: 8.0 },
            defence: DefenceComponent { slashing: 2.0 },
            hostile: false,
        }
    }
}

impl Renderable for NPC {
    fn render(&self, ctx: &mut RenderContext) {
        ctx.rectangle_fill([self.xy.0 as f64 * 16.0, self.xy.1 as f64 * 16.0, 16.0, 16.0], Color::from_hex("ff0000"));
    }
}

pub struct ActionPointsComponent {
    action_points: i32,
    max_action_points: u16,
}

impl ActionPointsComponent {
    fn new(max_ap: u16) -> ActionPointsComponent {
        ActionPointsComponent {
            action_points: max_ap as i32,
            max_action_points: max_ap
        }
    }

    pub fn can_use(&self, ap: u16) -> bool {
        return self.action_points >= ap as i32;
    }

    pub fn consume(&mut self, ap: u16) {
        self.action_points -= ap as i32;
    }

    pub fn fill(&mut self) {
        self.action_points = self.max_action_points as i32;
    }

}

pub struct HealthPointsComponent {
    health_points: f32,
    max_health_points: u16,
}

impl HealthPointsComponent {
    fn new(max_hp: f32) -> HealthPointsComponent {
        HealthPointsComponent {
            health_points: max_hp,
            max_health_points: max_hp as u16
        }
    }

    pub fn damage(&mut self, damage: f32) {
        self.health_points = (self.health_points - damage).max(0.0);
    }
}

pub struct DamageComponent {
    slashing: f32
}

impl DamageComponent {
    pub fn resolve(&self, defence: &DefenceComponent) -> f32 {
        return (self.slashing - defence.slashing).max(0.0)
    }
}

pub struct DefenceComponent {
    slashing: f32
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
