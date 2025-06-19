use crate::{commons::{damage_model::DamageComponent, resource_map::ResourceMap, rng::Rng}, engine::{animation::Animation, asset::image::ImageAsset, audio::SoundEffect, geometry::Coord2, Palette}, game::{actor::{actor::ActorType, damage_resolver::{resolve_damage, DamageOutput}, health_component::BodyPart}, chunk::{ChunkMap, TileMetadata}, effect_layer::EffectLayer, game_log::{GameLog, GameLogEntry}}, world::{item::ItemId, world::World}, Actor, EquipmentType, GameContext, GameSceneState, Item};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct ActionId(usize);
impl crate::commons::id_vec::Id for ActionId {
    fn new(id: usize) -> Self {
        ActionId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Actions = ResourceMap<ActionId, Action>;

#[derive(Clone)]
pub(crate) struct Action {
    pub(crate) name: String,
    pub(crate) icon: ImageAsset,
    pub(crate) description: String,
    pub(crate) sound_effect: Option<SoundEffect>,
    pub(crate) ap_cost: u16,
    pub(crate) stamina_cost: f32,
    pub(crate) action_type: ActionType
}

#[derive(Clone)]
pub(crate) enum ActionType {
    Move { offset: Coord2 },
    Targeted {
        damage: Option<DamageType>,
        inflicts: Option<Infliction>
    },
    Inspect,
    Dig,
    PickUp,
    Sleep
}

#[derive(Clone, Debug)]
pub(crate) enum DamageType {
    FromWeapon(DamageComponent),
    Fixed(DamageComponent)
}

#[derive(Clone)]
pub(crate) struct Infliction {
    pub(crate) chance: AfflictionChance,
    pub(crate) affliction: Affliction,
}

#[derive(Clone)]
pub(crate) enum AfflictionChance {
    OnHit
}

#[derive(Clone)]
pub(crate) enum Affliction {
    Bleeding { duration: usize },
    Poisoned { duration: usize },
    Stunned { duration: usize }
}

impl Affliction {
    pub(crate) fn name_color(&self) -> (&str, Palette) {
        match self {
            Affliction::Bleeding { duration: _ } => ("Bleeding", Palette::Red),
            Affliction::Poisoned { duration: _ } => ("Poisoned", Palette::Green),
            Affliction::Stunned { duration: _ } => ("Stunned", Palette::Gray),
        }
    }
}

pub(crate) struct ActionRunner { }

impl ActionRunner {
    pub(crate) fn move_try_use(action: &Action, actor: &mut Actor, chunk_map: &ChunkMap, ctx: &GameContext, player_pos: &Coord2) -> bool {
        match &action.action_type {
            ActionType::Move { offset } => {
                let ap_cost = (action.ap_cost as f32 * actor.stats().walk_ap_multiplier()) as u16;
                if actor.ap.can_use(ap_cost) && actor.stamina.can_use(action.stamina_cost) {
                    let xy = actor.xy.clone();
                    let pos = xy + *offset;
                    if !chunk_map.blocks_movement(pos) {
                        actor.ap.consume(ap_cost);
                        actor.stamina.consume(action.stamina_cost);
                        actor.xy = pos;
                        actor.animation.play(&Self::build_walk_anim());
                        if let Some(sound) = chunk_map.get_step_sound(xy) {
                            // TODO: Use actual camera
                            ctx.audio.play_positional(sound, xy.to_vec2(), player_pos.to_vec2());
                        }
                        return true
                    }
                }
            }
            _ => ()
        }
        return false
    }

    pub(crate) fn can_use(action: &Action, action_params: &ActionParams) -> Result<(), ActionFailReason> {
        if !action_params.actor.ap.can_use(action.ap_cost) {
            return Err(ActionFailReason::NotEnoughAP);
        }
        if !action_params.actor.stamina.can_use(action.stamina_cost) {
            return Err(ActionFailReason::NotEnoughStamina);
        }
        if action_params.actor.xy.dist_squared(&action_params.cursor_pos) >= 3. {
            return Err(ActionFailReason::CantReach);
        }
        match action.action_type {
            ActionType::Inspect => (),
            ActionType::Targeted { damage: _, inflicts: _ } => {
                if action_params.target.is_none() {
                    return Err(ActionFailReason::NoValidTarget);
                }
            },
            ActionType::PickUp => {
                if action_params.item_on_ground.is_none() {
                    return Err(ActionFailReason::NoValidTarget);
                }
            },
            ActionType::Dig => {
                if action_params.tile_metadata.is_none() {
                    return Err(ActionFailReason::NoValidTarget);
                }
            },
            ActionType::Sleep =>  {
                // TODO: Bed
                if action_params.object_tile != 3 {
                    return Err(ActionFailReason::NoValidTarget);
                }
            },
            // TODO:
            ActionType::Move { offset:_ } => (),
        }
        return Ok(());
    }

    pub(crate) fn try_use(action: &Action, action_params: &mut ActionUseParams, effect_layer: &mut EffectLayer, game_log: &mut GameLog, ctx: &GameContext) -> Result<Vec<ActionSideEffect>, ActionFailReason> {
        let r = Self::can_use(action, &action_params.as_simple_params());
        if let Err(reason) = r {
            return Err(reason);
        }
        drop(r);

        match action.action_type {
            ActionType::Inspect => {

                // TODO(hu2htwck): Add info to codex

                println!("Inspect at {:?}", action_params.cursor_pos);
                if let Some((_, target)) = &action_params.target {
                    let creature_id = target.creature_id;
                    if let Some(creature_id) = creature_id {
                        let codex = action_params.world.codex.creature_mut(&creature_id);
                        // TODO(hu2htwck): Not this
                        codex.add_appearance();
                        codex.add_name();
                        let creature = action_params.world.creatures.get(&creature_id);
                        println!("Target: {}, {:?}, {:?} birth {}", creature.name(&creature_id, &action_params.world, &ctx.resources), creature.profession, creature.gender, creature.birth.year());
                    }
                }
                if let Some((_, item)) = &action_params.item_on_ground {
                    println!("{}", item.description(&ctx.resources, &action_params.world));
                }
                let tile = action_params.object_tile;
                let tile_meta = &action_params.tile_metadata;
                match tile {
                    1 => println!("A wall."),
                    2 => println!("A tree."),
                    3 => println!("A bed."),
                    4 => println!("A table."),
                    5 => println!("A stool."),
                    6 => println!("A tombstone."),            
                    _ => ()                                
                };

                if let Some(meta) = tile_meta {
                    match meta {
                        TileMetadata::BurialPlace(creature_id) => {
                            let creature = action_params.world.creatures.get(creature_id);
                            if let Some(death) = creature.death {
                                let codex = action_params.world.codex.creature_mut(&creature_id);
                                codex.add_name();
                                codex.add_death();
                                // TODO(hu2htwck): Event
                                println!("The headstone says: \"Resting place of {:?}\". {} - {}. Died from {:?}", creature_id, creature.birth.year(), death.0.year(), death.1);
                            }
                            
                        }
                    }
                }
            },
            ActionType::Targeted { damage: _, inflicts: _ } => {
                if let Some((i, target)) = &mut action_params.target {
                    if ActionRunner::targeted_try_use(action, &mut action_params.actor, target, effect_layer, game_log, &action_params.world, ctx) {
                        let mut side_effects = Vec::new();
                        if target.hp.health_points() == 0. {
                            action_params.actor.add_xp(100);
                            side_effects.push(ActionSideEffect::RemoveNpc(*i));
                        }
                        if target.actor_type != ActorType::Player {
                            side_effects.push(ActionSideEffect::MakeNpcsHostile);
                        }
                        return Ok(side_effects);
                    }
                }
            },
            ActionType::PickUp => {
                let (i, item) = action_params.item_on_ground.as_mut().expect("msg");
                if let Ok(_) = action_params.actor.inventory.add(item.clone()) {
                    return Ok(vec!(ActionSideEffect::RemoveItemOnGround(*i)))
                }
            },
            ActionType::Dig => {
                if let Some(meta) = &mut action_params.tile_metadata {
                    match meta {
                        TileMetadata::BurialPlace(creature_id) => {
                            let mut side_effects = Vec::new();
                            side_effects.push(ActionSideEffect::RemoveObject(action_params.cursor_pos));
                            let creature = action_params.world.creatures.get(creature_id);
                            if let Some(details) = &creature.details {
                                for item in details.inventory.iter() {
                                    side_effects.push(ActionSideEffect::AddArtifactOnGround(action_params.cursor_pos, *item));
                                }
                            }
                            return Ok(side_effects)
                        }
                    }
                }
            },
            ActionType::Sleep =>  {
                // TODO: This healing doesn't work anymore.
                // self.chunk.player.hp.refill();
            },
            ActionType::Move { offset:_ } => (),
        }

        return Ok(vec!());
    }

    pub(crate) fn targeted_try_use(action: &Action, actor: &mut Actor, target: &mut Actor, effect_layer: &mut EffectLayer, game_log: &mut GameLog, world: &World, ctx: &GameContext) -> bool {
        match &action.action_type {
            ActionType::Targeted { damage, inflicts } => {
                if actor.ap.can_use(action.ap_cost) && actor.stamina.can_use(action.stamina_cost) {
                    if actor.xy.dist_squared(&target.xy) < 3. {
                        actor.ap.consume(action.ap_cost);
                        actor.stamina.consume(action.stamina_cost);
                        let mut hit = true;
                        if let Some(damage) = damage {
                            // Compute damage
                            let damage = match &damage {
                                DamageType::Fixed(dmg) => dmg.clone(),
                                DamageType::FromWeapon(dmg) => {
                                    let item = actor.inventory.equipped(&EquipmentType::Hand).expect("Used equipped action with no equipped item");
                                    let damage = dmg.multiply(item.damage_mult());
                                    let damage = damage + item.extra_damage(&ctx.resources.materials);
                                    damage
                                }
                            };
                            let target_body_part = BodyPart::random(&mut Rng::rand());
                            let damage = resolve_damage(&damage, &actor.stats(), &target_body_part, &target.stats());

                            match damage {
                                DamageOutput::Dodged => {
                                    hit = false;
                                    effect_layer.add_text_indicator(target.xy, "Dodged", Palette::Gray);
                                },
                                DamageOutput::Hit(damage) => {
                                    target.hp.hit(target_body_part, damage);
                                    effect_layer.add_damage_number(target.xy, damage);
                                },
                                DamageOutput::CriticalHit(damage) => {
                                    target.hp.critical_hit(target_body_part, damage);
                                    effect_layer.add_damage_number(target.xy, damage);
                                },
                            }

                            game_log.log(GameLogEntry::damage(actor, target, &damage, &world, &ctx.resources));

                            if let Some(fx) = &action.sound_effect {
                                ctx.audio.play_once(fx.clone());
                            }
                            // Animations
                            let dir = target.xy - actor.xy;
                            actor.animation.play(&Self::build_attack_anim(dir));
                            target.animation.play(&&Self::build_hurt_anim(dir));
                        }
                        if let Some(inflicts) = inflicts {
                            let inflict = match inflicts.chance {
                                AfflictionChance::OnHit => hit
                            };
                            if inflict {
                                let (name, color) = inflicts.affliction.name_color();
                                effect_layer.add_text_indicator(target.xy, name, color);
                                target.add_affliction(&inflicts.affliction)
                            }
                        }
                        return true
                    }
                }
            }
            _ => ()
        }
        return false
    }

    fn build_walk_anim() -> Animation {
        Animation::new()
            .translate(0.08, [0., -6.], crate::engine::animation::Smoothing::EaseInOut)
            .translate(0.08, [0., 0.], crate::engine::animation::Smoothing::EaseInOut)

    }

    fn build_hurt_anim(direction: Coord2) -> Animation {
        let direction = direction.to_vec2().normalize(12.);
        Animation::new()
            .translate(0.02, [direction.x as f64, direction.y as f64], crate::engine::animation::Smoothing::EaseInOut)
            .translate(0.2, [0., 0.], crate::engine::animation::Smoothing::EaseInOut)
    }

    fn build_attack_anim(direction: Coord2) -> Animation {
        let direction = direction.to_vec2().normalize(24.);
        Animation::new()
            .translate(0.08, [direction.x as f64, direction.y as f64], crate::engine::animation::Smoothing::EaseInOut)
            .translate(0.08, [0., 0.], crate::engine::animation::Smoothing::EaseInOut)
    }
}

pub(crate) struct ActionParams<'a> {
    pub(crate) actor: &'a Actor,
    pub(crate) cursor_pos: Coord2,
    pub(crate) target: Option<&'a Actor>,
    pub(crate) item_on_ground: Option<&'a Item>,
    pub(crate) tile_metadata: Option<&'a TileMetadata>,
    pub(crate) object_tile: usize
}

pub(crate) struct ActionUseParams<'a> {
    pub(crate) actor: &'a mut Actor,
    pub(crate) cursor_pos: Coord2,
    pub(crate) target: Option<(usize, &'a mut Actor)>,
    pub(crate) item_on_ground: Option<(usize, &'a Item)>,
    pub(crate) tile_metadata: Option<&'a TileMetadata>,
    pub(crate) object_tile: usize,
    pub(crate) world: &'a mut World,
}

impl<'a> ActionUseParams<'a> {

    fn as_simple_params(&'a self) -> ActionParams<'a> {
        ActionParams {
            actor: &self.actor,
            cursor_pos: self.cursor_pos,
            target: match &self.target {
                None => None,
                Some((_, v)) => Some(v)
            },
            item_on_ground: match &self.item_on_ground {
                None => None,
                Some((_, v)) => Some(v)
            },
            tile_metadata: match &self.tile_metadata {
                None => None,
                Some(v) => Some(v)
            },
            object_tile: self.object_tile,
        }
    }

}

#[derive(Debug)]
pub(crate) enum ActionFailReason {
    NotEnoughAP,
    NotEnoughStamina,
    CantReach,
    NoValidTarget
}

#[derive(Debug)]
pub(crate) enum ActionSideEffect {
    RemoveNpc(usize),
    MakeNpcsHostile,
    RemoveItemOnGround(usize),
    RemoveObject(Coord2),
    AddArtifactOnGround(Coord2, ItemId),
}

impl ActionSideEffect {

    pub(crate) fn run(&self, game: &mut GameSceneState, ctx: &mut GameContext) {
        match self {
            Self::RemoveNpc(i) => game.remove_npc(*i, ctx),
            Self::MakeNpcsHostile => {
                for p in game.chunk.npcs.iter_mut() {
                    p.actor_type = ActorType::Hostile;
                }
            },
            Self::RemoveItemOnGround(i) => {
                game.chunk.items_on_ground.remove(*i);
            },
            Self::RemoveObject(pos) => {
                game.chunk.map.remove_object(*pos);
                game.chunk.tiles_metadata.remove(pos);
            },
            Self::AddArtifactOnGround(pos, item) => {
                let item = game.world.artifacts.get(item);
                game.chunk.items_on_ground.push((*pos, item.clone(), item.make_texture(&ctx.resources.materials)));
            }
        }
    }

}