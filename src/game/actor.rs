use crate::{commons::{damage_model::DefenceComponent, history_vec::Id}, engine::{geometry::Coord2, render::RenderContext}, world::{attributes::Attributes, species::{Species, SpeciesIntelligence}, world::World}, Person};

use super::{inventory::inventory::Inventory, Renderable};

pub enum ActorType {
    Player,
    Passive,
    Hostile
}

pub struct Actor {
    pub xy: Coord2,
    pub ap: ActionPointsComponent,
    pub hp: HealthPointsComponent,
    pub attributes: Attributes,
    pub defence: DefenceComponent,
    pub actor_type: ActorType,
    pub texture: String,
    pub person_id: Option<Id>,
    pub person: Option<Person>,
    pub xp: u32,
    pub level: u32,
    pub inventory: Inventory
}

impl Actor {

    pub fn player(xy: Coord2, species: &Species) -> Actor {
        Actor {
            xy,
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthPointsComponent::new(&species.attributes),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0.),
            xp: 0,
            level: 1,
            person: None,
            person_id: None,
            texture: String::from("player.png"),
            actor_type: ActorType::Player,
            inventory: Inventory::new()
        }
    }

    pub fn from_species(xy: Coord2, species: &Species) -> Actor {
        let mut actor_type = ActorType::Passive;
        if species.intelligence == SpeciesIntelligence::Instinctive {
            actor_type = ActorType::Hostile;
        }
        Actor {
            xy,
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthPointsComponent::new(&species.attributes),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0.),
            xp: 0,
            level: 1,
            person: None,
            person_id: None,
            texture: species.texture.clone(),
            actor_type,
            inventory: Inventory::new()
        }
    }

    pub fn from_person(xy: Coord2, person_id: Id, person: &Person, species: &Species, world: &World) -> Actor {
        let mut actor_type = ActorType::Passive;
        if species.intelligence == SpeciesIntelligence::Instinctive {
            actor_type = ActorType::Hostile;
        }
        let mut inventory = Inventory::new();
        for id in person.possesions.iter() {
            let item = world.artifacts.get(id);
            inventory.add(item.clone());
            inventory.equip(0);
        }
        Actor {
            xy,
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthPointsComponent::new(&species.attributes),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0.),
            xp: 0,
            level: 1,
            person: Some(person.clone()),
            person_id: Some(person_id),
            texture: species.texture.clone(),
            actor_type,
            inventory
        }
    }

    pub fn update(&mut self) {
        self.hp.update(&self.attributes);
        self.ap.update(&self.attributes);
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
            self.attributes.unallocated += 1;
        }
    }
}

impl Renderable for Actor {
    fn render(&self, ctx: &mut RenderContext) {
        ctx.image(&self.texture, [self.xy.x as f64 * 24.0, self.xy.y as f64 * 24.0 - 8.]);
    }
}

pub struct ActionPointsComponent {
    pub action_points: i32,
    pub max_action_points: u16,
}

impl ActionPointsComponent {

    pub fn new(attributes: &Attributes) -> ActionPointsComponent {
        let max_ap = Self::max_ap(attributes);
        ActionPointsComponent {
            action_points: max_ap as i32,
            max_action_points: max_ap
        }
    }

    pub fn update(&mut self, attributes: &Attributes) {
        self.max_action_points = Self::max_ap(attributes);
        self.action_points = self.action_points.min(self.max_action_points as i32)
    }

    fn max_ap(attributes: &Attributes) -> u16 {
        let ap = 100 + attributes.bonus_ap();
        let ap = ap.clamp(0, u16::MAX as i32);
        return ap as u16
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

    pub fn new(attributes: &Attributes) -> HealthPointsComponent {
        let max_hp = Self::max_hp(attributes);
        HealthPointsComponent {
            health_points: max_hp as f32,
            max_health_points: max_hp
        }
    }

    pub fn refill(&mut self) {
        self.health_points = self.max_health_points as f32;
    }

    pub fn update(&mut self, attributes: &Attributes) {
        self.max_health_points = Self::max_hp(attributes);
        self.health_points = self.health_points.min(self.max_health_points as f32)
    }

    fn max_hp(attributes: &Attributes) -> u16 {
        let hp = 10 + attributes.bonus_hp();
        let hp = hp.clamp(0, u16::MAX as i32);
        return hp as u16
    }

    pub fn damage(&mut self, damage: f32) {
        self.health_points = (self.health_points - damage).max(0.0);
    }
}