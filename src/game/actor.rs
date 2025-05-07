use std::collections::HashMap;

use crate::{commons::{damage_model::DefenceComponent, rng::Rng}, engine::{animation::AnimationTransform, geometry::Coord2, render::RenderContext}, resources::{action::Affliction, species::{CreatureAppearance, Species, SpeciesId, SpeciesIntelligence}}, world::{attributes::Attributes, creature::{Creature, CreatureId, Profession}, world::World}, GameContext, Resources};

use super::{ai::AiRunner, effect_layer::EffectLayer, factory::item_factory::ItemFactory, health_component::HealthComponent, inventory::inventory::Inventory, Renderable};

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum ActorType {
    Player,
    Passive,
    Hostile
}

#[derive(Clone)]
pub(crate) struct Actor {
    pub(crate) xy: Coord2,
    pub(crate) animation: AnimationTransform,
    pub(crate) ap: ActionPointsComponent,
    pub(crate) hp: HealthComponent,
    pub(crate) attributes: Attributes,
    pub(crate) defence: DefenceComponent,
    pub(crate) actor_type: ActorType,
    pub(crate) ai: AiRunner,
    pub(crate) sprite: CreatureAppearance,
    pub(crate) creature_id: Option<CreatureId>,
    pub(crate) species: SpeciesId,
    pub(crate) xp: u32,
    pub(crate) level: u32,
    pub(crate) inventory: Inventory,
    afflictions: Vec<RunningAffliction>
}

impl Actor {

    pub(crate) fn player(xy: Coord2, species_id: &SpeciesId, species: &Species) -> Actor {
        Actor {
            xy,
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthComponent::new(),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0., species.attributes.dodge_chance()),
            xp: 0,
            level: 1,
            ai: AiRunner::new(),
            species: *species_id,
            creature_id: None,
            sprite: species.appearance.collapse(&Rng::rand(), &HashMap::new()),
            actor_type: ActorType::Player,
            inventory: Inventory::new(),
            afflictions: Vec::new()
        }
    }

    pub(crate) fn from_species(xy: Coord2, species_id: &SpeciesId, species: &Species) -> Actor {
        let mut actor_type = ActorType::Passive;
        if species.intelligence == SpeciesIntelligence::Instinctive {
            actor_type = ActorType::Hostile;
        }
        Actor {
            xy,
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthComponent::new(),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0., species.attributes.dodge_chance()),
            xp: 0,
            level: 1,
            ai: AiRunner::new(),
            species: *species_id,
            creature_id: None,
            sprite: species.appearance.collapse(&Rng::rand(), &HashMap::new()),
            actor_type,
            inventory: Inventory::new(),
            afflictions: Vec::new()
        }
    }

    pub(crate) fn from_creature(xy: Coord2, creature_id: CreatureId, creature: &Creature, species_id: &SpeciesId, species: &Species, world: &World, resources: &Resources) -> Actor {
        let mut actor_type = ActorType::Passive;
        if species.intelligence == SpeciesIntelligence::Instinctive {
            actor_type = ActorType::Hostile;
        }
        let mut inventory = Inventory::new();
        if let Some(details) = &creature.details {
            for id in details.inventory.iter() {
                let item = world.artifacts.get(id);
                inventory.add(item.clone());
                inventory.equip(0);
            }
        }

        if creature.profession == Profession::Guard || creature.profession == Profession::Bandit {
            let mut rng = Rng::seeded(creature_id);
            let item = ItemFactory::weapon(&mut rng, &resources);
            inventory.add(item);
            inventory.equip(0);
        }

        let mut hints = HashMap::new();
        hints.insert(String::from("clothes"), String::from("peasant"));
        match creature.profession {
            Profession::Guard => { hints.insert(String::from("clothes"), String::from("armor")); },
            Profession::Bandit => { hints.insert(String::from("clothes"), String::from("armor")); },
            // TODO:
            // Profession::Blacksmith => { hints.insert(String::from("clothes"), String::from("")); },
            // Profession::Sculptor => { hints.insert(String::from("clothes"), String::from("armor")); },
            // Profession::Ruler => { hints.insert(String::from("clothes"), String::from("armor")); },
            _ => (),
        }

        Actor {
            xy,
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(&species.attributes),
            hp: HealthComponent::new(),
            attributes: species.attributes.clone(),
            defence: DefenceComponent::new(0., 0., 0., species.attributes.dodge_chance()),
            xp: 0,
            level: 1,
            ai: AiRunner::new(),
            species: *species_id,
            creature_id: Some(creature_id),
            // TODO:
            //sprite: species.appearance.collapse(&Rng::rand(), &creature.appearance_hints),
            sprite: species.appearance.collapse(&Rng::rand(), &hints),
            actor_type,
            inventory,
            afflictions: Vec::new()
        }
    }

    pub(crate) fn update(&mut self, delta: f64) {
        self.animation.update(delta);
        // TODO: Why do this everytime?
        // self.hp.update(&self.attributes);
        self.ap.update(&self.attributes);
    }

    pub(crate) fn start_of_round(&mut self, effect_layer: &mut EffectLayer) {
        for affliction in self.afflictions.iter_mut() {
            affliction.delta += 1;
            match affliction.affliction {
                Affliction::Bleeding { duration: _ } => {
                    // TODO: Rethink
                    // self.hp.damage(1.);
                    effect_layer.add_damage_number(self.xy, 1.);
                },
                Affliction::Poisoned { duration: _ } => {
                    // TODO: Rethink
                    // self.hp.damage(1.);
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

    pub(crate) fn add_affliction(&mut self, affliction: &Affliction) {
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

    pub(crate) fn add_xp(&mut self, ammount: u32) {
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
pub(crate) struct ActionPointsComponent {
    pub(crate) action_points: i32,
    pub(crate) max_action_points: u16,
}

impl ActionPointsComponent {

    pub(crate) fn new(attributes: &Attributes) -> ActionPointsComponent {
        let max_ap = Self::max_ap(attributes);
        ActionPointsComponent {
            action_points: max_ap as i32,
            max_action_points: max_ap
        }
    }

    pub(crate) fn update(&mut self, attributes: &Attributes) {
        self.max_action_points = Self::max_ap(attributes);
        self.action_points = self.action_points.min(self.max_action_points as i32)
    }

    fn max_ap(attributes: &Attributes) -> u16 {
        let ap = 100 + attributes.bonus_ap();
        let ap = ap.clamp(0, u16::MAX as i32);
        return ap as u16
    }

    pub(crate) fn can_use(&self, ap: u16) -> bool {
        return self.action_points >= ap as i32;
    }

    pub(crate) fn consume(&mut self, ap: u16) {
        self.action_points -= ap as i32;
    }

    pub(crate) fn fill(&mut self) {
        self.action_points = self.max_action_points as i32;
    }

}

#[derive(Clone)]
struct RunningAffliction {
    affliction: Affliction,
    delta: usize,
}