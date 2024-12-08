use crate::{commons::history_vec::Id, engine::{geometry::Coord2, render::RenderContext}, Person};

use super::Renderable;

pub enum ActorType {
    Player,
    Passive,
    Hostile
}

pub struct Actor {
    pub xy: Coord2,
    pub ap: ActionPointsComponent,
    pub hp: HealthPointsComponent,
    pub damage: DamageComponent,
    pub defence: DefenceComponent,
    pub actor_type: ActorType,
    pub texture: String,
    pub person_id: Option<Id>,
    pub person: Option<Person>,
    pub xp: u32,
    pub level: u32,
}

impl Actor {
    pub fn new(xy: Coord2, actor_type: ActorType, person_id: Option<Id>, person: Option<&Person>) -> Actor {
        let texture;
        if let ActorType::Player = actor_type {
            texture = "player.png";
        } else {
            if let Some(_) = person_id {
                texture = "character.png";
            } else {
                texture = "spider.png";
            }
        }
        let person_clone;
        if let Some(person) = person {
            person_clone = Some(person.clone());
        } else {
            person_clone = None;
        }
        Actor {
            xy,
            ap: ActionPointsComponent::new(100),
            hp: HealthPointsComponent::new(100.),
            damage: DamageComponent { slashing: 10.0 },
            defence: DefenceComponent { slashing: 3.0 },
            xp: 0,
            level: 1,
            person: person_clone,
            person_id,
            texture: String::from(texture),
            actor_type
        }
    }

    pub fn add_xp(&mut self, ammount: u32) {
        self.xp += ammount;
        let mut new_level = 1;
        if self.xp >= 300 {
            new_level = 2;
        }
        if self.xp >= 900 {
            new_level = 3;
        }
        if self.xp >= 2700 {
            new_level = 4;
        }
        if self.xp >= 6500 {
            new_level = 5;
        }
        if self.xp >= 14000 {
            new_level = 6;
        }
        if self.xp >= 23000 {
            new_level = 7;
        }
        if self.xp >= 34000 {
            new_level = 8;
        }
        if self.xp >= 48000 {
            new_level = 9;
        }
        if self.xp >= 64000 {
            new_level = 10;
        }
        if self.xp >= 85000 {
            new_level = 11;
        }
        if self.xp >= 100000 {
            new_level = 12;
        }
        if self.xp >= 120000 {
            new_level = 13;
        }
        if self.xp >= 140000 {
            new_level = 14;
        }
        if self.xp >= 165000 {
            new_level = 15;
        }
        if self.xp >= 195000 {
            new_level = 16;
        }
        if self.xp >= 225000 {
            new_level = 17;
        }
        if self.xp >= 265000 {
            new_level = 18;
        }
        if self.xp >= 305000 {
            new_level = 19;
        }
        if self.xp >= 355000 {
            new_level = 20;
        }
        if new_level > self.level {
            self.level = new_level;
            self.damage.slashing += 1.;
            self.defence.slashing += 0.2;
        }
    }
}

impl Renderable for Actor {
    fn render(&self, ctx: &mut RenderContext) {
        ctx.image(&self.texture, [self.xy.x as f64 * 16.0, self.xy.y as f64 * 16.0]);
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