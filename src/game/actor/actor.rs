use graphics::Transformed;
use math::Vec2i;
use serde::{Deserialize, Serialize};

use crate::{commons::{interpolate::lerp, rng::Rng}, engine::{animation::AnimationTransform, assets::assets, geometry::{Coord2, Size2D}, render::RenderContext}, game::{actor::health_component::BodyPart, ai::{AiRunner, AiState}, effect_layer::EffectLayer, inventory::inventory::Inventory, Renderable}, resources::{action::{ActionId, Affliction}, species::{CreatureAppearance, LayerType, Species, SpeciesId}}, world::{attributes::Attributes, creature::{Creature, CreatureGender, CreatureId}, world::World}, EquipmentType, GameContext, Resources};

use super::{actor_stats::ActorStats, equipment_generator::EquipmentGenerator, health_component::HealthComponent};

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Actor {
    pub(crate) xy: Vec2i,
    #[serde(skip)]
    pub(crate) animation: AnimationTransform,
    pub(crate) ap: ActionPointsComponent,
    pub(crate) stamina: StaminaComponent,
    pub(crate) hp: HealthComponent,
    pub(crate) attributes: Attributes,
    pub(crate) ai_state: AiState,
    pub(crate) ai_group: u8,
    #[serde(skip)]
    pub(crate) ai: AiRunner,
    pub(crate) sprite_flipped: bool,
    pub(crate) sprite: CreatureAppearance,
    pub(crate) creature_id: Option<CreatureId>,
    pub(crate) gender: CreatureGender,
    pub(crate) species: SpeciesId,
    pub(crate) xp: u32,
    pub(crate) level: u32,
    pub(crate) inventory: Inventory,
    pub(crate) cooldowns: Vec<(ActionId, u16)>,
    pub(crate) afflictions: Vec<RunningAffliction>,
    age: i32,
    just_entered_fight: bool,
}

impl Actor {

    pub(crate) fn from_species(xy: Coord2, species_id: &SpeciesId, species: &Species, ai_group: u8) -> Actor {
        let gender = CreatureGender::random();
        Actor {
            xy: xy.to_vec2i(),
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(),
            stamina: StaminaComponent::new(),
            hp: HealthComponent::new(species.max_hp),
            attributes: species.attributes.clone(),
            xp: 0,
            level: 1,
            ai_state: AiState::Disabled,
            ai_group,
            ai: AiRunner::new(),
            species: *species_id,
            creature_id: None,
            gender,
            sprite_flipped: Rng::rand().rand_chance(0.5),
            sprite: species.appearance.collapse(&gender),
            inventory: Inventory::new(),
            afflictions: Vec::new(),
            cooldowns: Vec::new(),
            just_entered_fight: false,
            age: 20
        }
    }

    pub(crate) fn from_creature(xy: Coord2, ai_group: u8, creature_id: CreatureId, creature: &Creature, species_id: &SpeciesId, species: &Species, world: &World, resources: &Resources) -> Actor {
        let mut rng = Rng::seeded(creature_id);
        let inventory = match creature.sim_flag_is_inteligent() {
            true => EquipmentGenerator::generate(&creature_id, &mut rng, world, resources),
            false => Inventory::new()
        };
        Actor {
            xy: xy.to_vec2i(),
            animation: AnimationTransform::new(),
            ap: ActionPointsComponent::new(),
            stamina: StaminaComponent::new(),
            hp: HealthComponent::new(species.max_hp),
            attributes: species.attributes.clone(),
            xp: 0,
            level: 1,
            ai_state: AiState::Disabled,
            ai_group,
            ai: AiRunner::new(),
            species: *species_id,
            creature_id: Some(creature_id),
            gender: creature.gender,
            sprite_flipped: Rng::rand().rand_chance(0.5),
            sprite: species.appearance.collapse(&creature.gender),
            inventory,
            afflictions: Vec::new(),
            cooldowns: Vec::new(),
            just_entered_fight: false,
            age: (world.date - creature.birth).get_years()
        }
    }

    pub(crate) fn update(&mut self, delta: f64) {
        self.animation.update(delta);
    }

    pub(crate) fn start_of_round(&mut self, effect_layer: &mut EffectLayer) {
        for affliction in self.afflictions.iter_mut() {
            affliction.remaining -= 1;
            match affliction.affliction {
                Affliction::Bleeding { duration: _ } => {
                    let target_body_part = BodyPart::random(&mut Rng::rand());
                    self.hp.hit(target_body_part, 5.);
                    effect_layer.add_damage_number(self.xy.into(), 5.);
                },
                Affliction::OnFire { duration: _ } => {
                    let target_body_part = BodyPart::random(&mut Rng::rand());
                    self.hp.hit(target_body_part, 5.);
                    effect_layer.add_damage_number(self.xy.into(), 5.);
                },
                Affliction::Poisoned { duration: _ } => {
                    // TODO: Rethink
                    // self.hp.damage(1.);
                    effect_layer.add_damage_number(self.xy.into(), 1.);
                },
                Affliction::Stunned { duration: _ } => {
                    self.ap.consume(self.ap.max_action_points / 4);
                },
                Affliction::Healing { duration: _, strength } => {
                    self.hp.heal(strength);
                },
                Affliction::Recovery { duration: _, strength } => {
                    self.hp.recover(strength);
                },
            }
        }
        self.afflictions.retain(|affliction| affliction.remaining > 0);

        self.cooldowns.retain_mut(|cooldown| {
            cooldown.1 -= 1;
            return cooldown.1 > 0;
        });
        self.just_entered_fight = false;
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

        let duration = match affliction {
            Affliction::Bleeding { duration } => *duration,
            Affliction::Poisoned { duration } => *duration,
            Affliction::OnFire { duration } => *duration,
            Affliction::Healing { duration, strength: _ } => *duration,
            Affliction::Stunned { duration } => *duration,
            Affliction::Recovery { duration, strength: _ } => *duration,
        };

        self.afflictions.push(RunningAffliction { affliction: affliction.clone(), remaining: duration as i32 });
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

        vec.push(game_ctx.resources.actions.id_of("act:talk"));
        vec.push(game_ctx.resources.actions.id_of("act:inspect"));
        vec.push(game_ctx.resources.actions.id_of("act:pickup"));
        vec.push(game_ctx.resources.actions.id_of("act:dig"));
        vec.push(game_ctx.resources.actions.id_of("act:sleep"));
        vec.push(game_ctx.resources.actions.id_of("act:harvest"));
        
        return vec;
    }

    pub(crate) fn set_ai_state(&mut self, ai_state: AiState) {
        if ai_state == AiState::Fight && self.ai_state != ai_state {
            self.just_entered_fight = true;
        }
        self.ai_state = ai_state;
    }

    pub(crate) fn render_layers(&self, pos: [f64; 2], ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        ctx.push();
        if self.age < 16 {
            let scale = lerp(0.5, 1.0, self.age as f64 / 16.);
            ctx.context.transform = ctx.context.transform
                .trans(24. * (1.-scale), 48. * (1.-scale))
                .scale(scale, scale);
        }
        for (sprite, layer_type) in self.sprite.textures().iter() {
            match layer_type {
                LayerType::Skin => {
                    sprite.draw(ctx.at(pos[0], pos[1]), ctx.gl);
                },
                LayerType::Hair => {
                    if self.age > 30 {
                        let whiteness = lerp(1., 2., (self.age - 30) as f64 / 20.).max(1.) as f32;
                        sprite.draw_colored(ctx.at(pos[0], pos[1]), [whiteness * 0.8, whiteness * 0.9, whiteness, 1.], ctx.gl);
                    } else {
                        sprite.draw(ctx.at(pos[0], pos[1]), ctx.gl);
                    } 
                }
            }
        }
        let mut textures = Vec::new();
        for (slot, item) in self.inventory.all_equipped() {
            let blueprint = game_ctx.resources.item_blueprints.get(&item.blueprint_id);
            if blueprint.equippable.is_some() {
                let z_order = match slot {
                    EquipmentType::Feet => 1,
                    EquipmentType::Legs => 2,
                    EquipmentType::TorsoGarment => 3,
                    EquipmentType::TorsoInner => 4,
                    EquipmentType::Head => 10,
                    EquipmentType::Hand => 200,
                    EquipmentType::Trinket => 201,
                };
                let index = match self.sprite {
                    CreatureAppearance::Single(_, _) => 0,
                    CreatureAppearance::Composite { index, base: _, top: _ } => index
                };
                textures.push((z_order, item.make_inventory_texture(index, &game_ctx.resources)));
            }
        }
        textures.sort_by(|a, b| a.0.cmp(&b.0));
        for (_z, image) in textures {
            ctx.texture_old(image, pos);
        }
        let _ = ctx.try_pop();
    }
}

impl Renderable for Actor {
    fn render(&self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let pos: [f64; 2] = [self.xy.x() as f64 * 24.0 - 12., self.xy.y() as f64 * 24.0 - 24.];

        ctx.push();
        ctx.context.transform = ctx.context.transform.trans(pos[0], pos[1]).trans_pos(self.animation.translate_no_z());

        if self.sprite_flipped {
            ctx.context.transform = ctx.context.transform.trans(48., 0.).scale(-1., 1.)
        }

        ctx.image("species/shadow.png", [11, 42]);

        ctx.context.transform = ctx.context.transform.trans_pos(self.animation.translate_z_only());

        self.render_layers([0., 0.], ctx, game_ctx);

        for affliction in self.afflictions.iter() {
            match affliction.affliction {
                Affliction::OnFire { duration: _ } => {
                    let sheet = assets().image_sheet("status/onfire.png", Size2D(24, 24));
                    ctx.texture(sheet.textures.get(ctx.sprite_i % sheet.len()).unwrap(), ctx.at(11., 24.));
                },
                Affliction::Stunned { duration: _ } => {
                    let sheet = assets().image_sheet("status/stunned.png", Size2D(24, 32));
                    ctx.texture(sheet.textures.get(ctx.sprite_i % sheet.len()).unwrap(), ctx.at(11., 16.));
                },
                _ => ()
            }
        }
        if self.just_entered_fight {
            let sheet = assets().image_sheet("status/entered_fight.png", Size2D(24, 32));
            ctx.texture(sheet.textures.get(ctx.sprite_i % sheet.len()).unwrap(), ctx.at(11., 16.));
        }
        let _ = ctx.try_pop();
    }
}

#[derive(Clone, Serialize, Deserialize)]
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


#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct RunningAffliction {
    pub(crate) affliction: Affliction,
    pub(crate) remaining: i32,
}