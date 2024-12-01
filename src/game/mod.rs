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
    turn_controller: TurnController
}

impl GameSceneState {
    pub fn new() -> GameSceneState {
        GameSceneState {
            player: Player::new(Point2D(32, 32)),
            npcs: vec!(
                NPC::new(Point2D(10, 10)),
                NPC::new(Point2D(50, 50)),
                NPC::new(Point2D(50, 10)),
                NPC::new(Point2D(10, 50)),
            ),
            turn_controller: TurnController { turn_idx: 0 }
        }
    }
}

impl Scene for GameSceneState {
    fn render(&self, mut ctx: RenderContext) {
        self.player.render(&mut ctx);
        for npc in self.npcs.iter() {
            if npc.hp.health_points > 0.0 {
                npc.render(&mut ctx);
            }
        }
        if self.turn_controller.is_player_turn() {
            let txt = format!("Player turn ({} / {})", self.player.ap.action_points, self.player.ap.max_action_points);
            ctx.text(txt.as_str(), 10, [10.0, 10.0], Color::from_hex("ffffff"));
        } else {
            let txt = format!("Enemy turn {}", self.turn_controller.npc_idx());
            ctx.text(txt.as_str(), 10, [10.0, 10.0], Color::from_hex("ffffff"));
        }
    }

    fn update(&mut self) {
        if self.turn_controller.is_player_turn() {
            return
        }
        println!("AI Turn: {}", self.turn_controller.npc_idx());
        let npc = self.npcs.get(self.turn_controller.npc_idx()).unwrap();
        // TODO: Actually remove the NPC
        if npc.hp.health_points == 0.0 {
            self.turn_controller.next_turn(self.npcs.len());
            return
        }
        // TODO: AI
        self.turn_controller.next_turn(self.npcs.len());
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
                    self.turn_controller.next_turn(self.npcs.len());
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
                        let target = self.npcs.iter_mut().find(|npc| npc.xy == tile_pos);
                        if let Some(target) = target {
                            println!("Attack! {:?}", tile_pos);
                            self.player.ap.consume(40);
                            target.hp.damage(self.player.damage.resolve(&target.defence));
                        } else {
                            println!("No target {:?}", tile_pos);
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
    pub defence: DefenceComponent
}

impl NPC {
    pub fn new(xy: Point2D) -> NPC {
        NPC {
            xy,
            ap: ActionPointsComponent::new(80),
            hp: HealthPointsComponent::new(50.),
            damage: DamageComponent { slashing: 10.0 },
            defence: DefenceComponent { slashing: 3.0 }
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
    turn_idx: usize
}

impl TurnController {

    pub fn is_player_turn(&self) -> bool {
        return self.turn_idx == 0
    }

    pub fn npc_idx(&self) -> usize {
        return self.turn_idx - 1
    }

    pub fn next_turn(&mut self, npcs_count: usize) {
        self.turn_idx = (self.turn_idx + 1) % (npcs_count + 1);
    }

}