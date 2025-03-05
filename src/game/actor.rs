use std::collections::HashMap;

use crate::{commons::{damage_model::DefenceComponent, history_vec::Id, rng::Rng}, engine::{animation::AnimationTransform, geometry::Coord2, render::RenderContext}, world::{attributes::Attributes, species::{CreatureAppearance, Species, SpeciesId, SpeciesIntelligence}, world::World}, GameContext, Person};

use super::{action::Affliction, ai::AiRunner, effect_layer::EffectLayer, inventory::inventory::Inventory, Renderable};

#[derive(Clone, PartialEq, Eq)]
pub enum ActorType {
    Player,
    Passive,
    Hostile
}

#[derive(Clone)]
pub struct Actor {
    pub xy: Coord2,
    pub animation: AnimationTransform,
    pub ap: ActionPointsComponent,
    pub hp: HealthPointsComponent,
    pub attributes: Attributes,
    pub defence: DefenceComponent,
    pub actor_type: ActorType,
    pub ai: AiRunner,
    pub sprite: CreatureAppearance,
    pub person_id: Option<Id>,
    pub person: Option<Person>,
    pub species: SpeciesId,
    pub xp: u32,
    pub level: u32,
    pub inventory: Inventory,
    afflictions: Vec<RunningAffliction>
}

impl Actor {

    pub fn player(xy: Coord2, species_id: &SpeciesId, species: &Species) -> Actor {
        Actor {
            xy,
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthPointsComponent::new(&species.attributes),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0., species.attributes.dodge_chance()),
            xp: 0,
            level: 1,
            ai: AiRunner::new(),
            species: *species_id,
            person: None,
            person_id: None,
            sprite: species.appearance.collapse(&Rng::rand(), &HashMap::new()),
            actor_type: ActorType::Player,
            inventory: Inventory::new(),
            afflictions: Vec::new()
        }
    }

    pub fn from_species(xy: Coord2, species_id: &SpeciesId, species: &Species) -> Actor {
        let mut actor_type = ActorType::Passive;
        if species.intelligence == SpeciesIntelligence::Instinctive {
            actor_type = ActorType::Hostile;
        }
        Actor {
            xy,
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthPointsComponent::new(&species.attributes),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0., species.attributes.dodge_chance()),
            xp: 0,
            level: 1,
            ai: AiRunner::new(),
            species: *species_id,
            person: None,
            person_id: None,
            sprite: species.appearance.collapse(&Rng::rand(), &HashMap::new()),
            actor_type,
            inventory: Inventory::new(),
            afflictions: Vec::new()
        }
    }

    pub fn from_person(xy: Coord2, person_id: Id, person: &Person, species_id: &SpeciesId, species: &Species, world: &World) -> Actor {
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
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthPointsComponent::new(&species.attributes),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0., species.attributes.dodge_chance()),
            xp: 0,
            level: 1,
            ai: AiRunner::new(),
            species: *species_id,
            person: Some(person.clone()),
            person_id: Some(person_id),
            sprite: species.appearance.collapse(&Rng::rand(), &person.appearance_hints),
            actor_type,
            inventory,
            afflictions: Vec::new()
        }
    }

    pub fn update(&mut self, delta: f64) {
        self.animation.update(delta);
        // TODO: Why do this everytime?
        self.hp.update(&self.attributes);
        self.ap.update(&self.attributes);
    }

    pub fn start_of_round(&mut self, effect_layer: &mut EffectLayer) {
        for affliction in self.afflictions.iter_mut() {
            affliction.delta += 1;
            match affliction.affliction {
                Affliction::Bleeding { duration: _ } => {
                    self.hp.damage(1.);
                    effect_layer.add_damage_number(self.xy, 1.);
                },
                Affliction::Poisoned { duration: _ } => {
                    self.hp.damage(1.);
                    effect_layer.add_damage_number(self.xy, 1.);
                },
                Affliction::Stunned { duration: _ } => {
                    self.ap.consume(self.ap.max_action_points / 4);
                }
            }
        }
        self.afflictions.retain(|affliction| {
            match affliction.affliction {
                Affliction::Bleeding { duration } => affliction.delta < duration,
                Affliction::Poisoned { duration } => affliction.delta < duration,
                Affliction::Stunned { duration } => affliction.delta < duration,
            }
        });
    }

    pub fn add_affliction(&mut self, affliction: &Affliction) {
        // Removes dupped
        self.afflictions.retain(|a| {
            let current = &a.affliction;
            match (current, affliction) {
                (Affliction::Bleeding { duration: _ }, Affliction::Bleeding { duration: _ }) => false,
                (Affliction::Poisoned { duration: _ }, Affliction::Poisoned { duration: _ }) => false,
                (Affliction::Stunned { duration: _ }, Affliction::Stunned { duration: _ }) => false,
                _ => true
            }
        });
        self.afflictions.push(RunningAffliction { affliction: affliction.clone(), delta: 0 });
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
    fn render(&self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let mut pos: [f64; 2] = [self.xy.x as f64 * 24.0 - 12., self.xy.y as f64 * 24.0 - 24.];
        // Applies the animation to the rendering
        pos[0] += self.animation.translate[0];
        pos[1] += self.animation.translate[1];
        let textures = self.sprite.texture();
        for texture in textures {
            ctx.texture(texture, pos);
        }
        let item = self.inventory.equipped();
        if let Some(item) = item {
            ctx.texture(item.make_equipped_texture(&game_ctx.resources.materials), pos);
        }
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
struct RunningAffliction {
    affliction: Affliction,
    delta: usize,
}