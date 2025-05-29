use std::collections::HashMap;

use crate::{commons::rng::Rng, engine::{animation::AnimationTransform, geometry::Coord2, render::RenderContext}, game::{ai::AiRunner, effect_layer::EffectLayer, inventory::inventory::Inventory, Renderable}, resources::{action::{ActionId, Affliction}, species::{CreatureAppearance, Species, SpeciesId, SpeciesIntelligence}}, world::{attributes::Attributes, creature::{Creature, CreatureId}, world::World}, GameContext, Resources};

use super::{actor_stats::ActorStats, equipment_generator::EquipmentGenerator, health_component::HealthComponent};

#[derive(Clone, Copy, PartialEq, Eq)]
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
    pub(crate) stamina: StaminaComponent,
    pub(crate) hp: HealthComponent,
    pub(crate) attributes: Attributes,
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
            ap: ActionPointsComponent::new(),
            stamina: StaminaComponent::new(),
            hp: HealthComponent::new(),
            attributes: species.attributes.clone(),
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
            ap: ActionPointsComponent::new(),
            stamina: StaminaComponent::new(),
            hp: HealthComponent::new(),
            attributes: species.attributes.clone(),
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
        // TODO: Determinate
        let mut rng = Rng::seeded(creature_id);
        let inventory = EquipmentGenerator::generate(&creature_id, &mut rng, world, resources);
       
        let mut hints = HashMap::new();
        if creature.gender.is_male() {
            hints.insert(String::from("base"), String::from("male_light")); 
        } else {
            hints.insert(String::from("base"), String::from("female_light")); 
        }

        Actor {
            xy,
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(),
            stamina: StaminaComponent::new(),
            hp: HealthComponent::new(),
            attributes: species.attributes.clone(),
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

    pub(crate) fn stats<'a>(&'a self) -> ActorStats<'a> {
        return ActorStats::new(&self)
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

    pub(crate) fn get_all_available_actions(&self, game_ctx: &mut GameContext) -> Vec<ActionId> {
        let mut vec = Vec::new();

        let species = game_ctx.resources.species.get(&self.species);
        vec.extend(species.innate_actions.clone());

        for (_slot, item) in self.inventory.all_equipped() {
            if let Some(action_provider) = &item.action_provider {
                vec.extend(action_provider.actions.clone());
            }
        }

        if let ActorType::Player = self.actor_type {
            vec.push(game_ctx.resources.actions.id_of("act:inspect"));
            vec.push(game_ctx.resources.actions.id_of("act:pickup"));
            vec.push(game_ctx.resources.actions.id_of("act:dig"));
            vec.push(game_ctx.resources.actions.id_of("act:sleep"));
        }
        
        return vec;
    }

    pub(crate) fn render_layers(&self, pos: [f64; 2], ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let textures = self.sprite.texture();
        for texture in textures {
            ctx.texture(texture, pos);
        }
        // TODO:
        let equipment = self.inventory.all_equipped();
        for (_slot, item) in equipment {
            if let Some(equippable) = &item.equippable {
                ctx.texture(equippable.make_texture(&item.material, &game_ctx.resources.materials), pos);
            }
        }
    }
}

impl Renderable for Actor {
    fn render(&self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let mut pos: [f64; 2] = [self.xy.x as f64 * 24.0 - 12., self.xy.y as f64 * 24.0 - 24.];
        // Applies the animation to the rendering
        pos[0] += self.animation.translate[0];
        pos[1] += self.animation.translate[1];
        self.render_layers(pos, ctx, game_ctx);
    }
}

#[derive(Clone)]
pub(crate) struct ActionPointsComponent {
    pub(crate) action_points: i32,
    pub(crate) max_action_points: u16,
}

impl ActionPointsComponent {

    pub(crate) fn new() -> ActionPointsComponent {
        let max_ap = Self::max_ap();
        ActionPointsComponent {
            action_points: max_ap as i32,
            max_action_points: max_ap
        }
    }

    fn max_ap() -> u16 {
        return 100
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
pub(crate) struct StaminaComponent {
    pub(crate) stamina: f32,
    pub(crate) max_stamina: f32,
}

impl StaminaComponent {

    pub(crate) fn new() -> StaminaComponent {
        StaminaComponent {
            stamina: 100.,
            max_stamina: 100.
        }
    }

    pub(crate) fn can_use(&self, stamina: f32) -> bool {
        return self.stamina >= stamina;
    }

    pub(crate) fn consume(&mut self, ap: f32) {
        self.stamina -= ap;
    }

    pub(crate) fn recover_turn(&mut self) {
        self.stamina = (self.stamina + 1.).min(self.max_stamina);
    }

}

#[derive(Clone)]
struct RunningAffliction {
    affliction: Affliction,
    delta: usize,
}