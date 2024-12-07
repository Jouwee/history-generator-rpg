use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::{commons::history_vec::Id, engine::{geometry::Coord2, render::RenderContext}, Person};

use super::Renderable;

pub trait Actor {
    fn xy(&self) -> Coord2;
    fn set_xy(&mut self, xy: Coord2);
    fn ap(&mut self) -> &mut ActionPointsComponent;
    fn hp(&mut self) -> &mut HealthPointsComponent;
    fn damage(&mut self) -> &mut DamageComponent;
    fn defence(&mut self) -> &mut DefenceComponent;
}

pub struct Player {
    pub xy: Coord2,
    pub ap: ActionPointsComponent,
    pub hp: HealthPointsComponent,
    pub damage: DamageComponent,
    pub defence: DefenceComponent
}

impl Player {
    pub fn new(xy: Coord2) -> Player {
        Player {
            xy,
            ap: ActionPointsComponent::new(100),
            hp: HealthPointsComponent::new(100.),
            damage: DamageComponent { slashing: 10.0 },
            defence: DefenceComponent { slashing: 3.0 }
        }
    }
}

impl Actor for Player {
    fn xy(&self) -> Coord2 {
        self.xy
    }
    fn set_xy(&mut self, xy: Coord2) {
        self.xy = xy
    }
    fn hp(&mut self) -> &mut HealthPointsComponent {
        &mut self.hp
    }
    fn ap(&mut self) -> &mut ActionPointsComponent {
        &mut self.ap
    }
    fn damage(&mut self) -> &mut DamageComponent {
        &mut self.damage
    }
    fn defence(&mut self) -> &mut DefenceComponent {
        &mut self.defence
    }
}

impl Renderable for Player {
    fn render(&self, ctx: &mut RenderContext) {
        ctx.image("player.png", [self.xy.x as f64 * 16.0, self.xy.y as f64 * 16.0]);
    }
}

pub struct NPC {
    pub xy: Coord2,
    pub ap: ActionPointsComponent,
    pub hp: HealthPointsComponent,
    pub damage: DamageComponent,
    pub defence: DefenceComponent,
    pub hostile: bool,
    pub texture: Texture,
    pub person_id: Id,
    pub person: Person
}

impl NPC {
    pub fn new(xy: Coord2, person_id: Id, person: &Person) -> NPC {
        let spritesheet = ImageReader::open("./assets/sprites/character.png").unwrap().decode().unwrap();
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let texture = Texture::from_image(&spritesheet.to_rgba8(), &settings);
        NPC {
            xy,
            ap: ActionPointsComponent::new(80),
            hp: HealthPointsComponent::new(50.),
            damage: DamageComponent { slashing: 8.0 },
            defence: DefenceComponent { slashing: 2.0 },
            hostile: false,
            texture,
            person_id,
            person: person.clone()
        }
    }
}

impl Renderable for NPC {
    fn render(&self, ctx: &mut RenderContext) {
        ctx.image("character.png", [self.xy.x as f64 * 16.0, self.xy.y as f64 * 16.0]);
    }
}

impl Actor for NPC {
    fn xy(&self) -> Coord2 {
        self.xy
    }
    fn set_xy(&mut self, xy: Coord2) {
        self.xy = xy
    }
    fn ap(&mut self) -> &mut ActionPointsComponent {
        &mut self.ap
    }
    fn hp(&mut self) -> &mut HealthPointsComponent {
        &mut self.hp
    }
    fn damage(&mut self) -> &mut DamageComponent {
        &mut self.damage
    }
    fn defence(&mut self) -> &mut DefenceComponent {
        &mut self.defence
    }
}


pub struct ActionPointsComponent {
    pub action_points: i32,
    pub max_action_points: u16,
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
    pub health_points: f32,
    pub max_health_points: u16,
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