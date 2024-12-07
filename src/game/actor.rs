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
    pub defence: DefenceComponent,
    pub xp: u32,
    pub level: u32,
}

impl Player {
    pub fn new(xy: Coord2) -> Player {
        Player {
            xy,
            ap: ActionPointsComponent::new(100),
            hp: HealthPointsComponent::new(100.),
            damage: DamageComponent { slashing: 10.0 },
            defence: DefenceComponent { slashing: 3.0 },
            xp: 0,
            level: 1,
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

impl Player {

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

pub struct NPC {
    pub xy: Coord2,
    pub ap: ActionPointsComponent,
    pub hp: HealthPointsComponent,
    pub damage: DamageComponent,
    pub defence: DefenceComponent,
    pub hostile: bool,
    pub texture: String,
    pub person_id: Option<Id>,
    pub person: Option<Person>
}

impl NPC {
    pub fn new(xy: Coord2, person_id: Option<Id>, person: Option<&Person>) -> NPC {
        let spritesheet;
        if let Some(_) = person_id {
            spritesheet = "character.png";
        } else {
            spritesheet = "spider.png";
        }
        let person_clone;
        if let Some(person) = person {
            person_clone = Some(person.clone());
        } else {
            person_clone = None;
        }
        NPC {
            xy,
            ap: ActionPointsComponent::new(80),
            hp: HealthPointsComponent::new(50.),
            damage: DamageComponent { slashing: 8.0 },
            defence: DefenceComponent { slashing: 2.0 },
            hostile: false,
            texture: String::from(spritesheet),
            person_id,
            person: person_clone
        }
    }
}

impl Renderable for NPC {
    fn render(&self, ctx: &mut RenderContext) {
        ctx.image(&self.texture, [self.xy.x as f64 * 16.0, self.xy.y as f64 * 16.0]);
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