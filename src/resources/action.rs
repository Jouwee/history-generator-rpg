use std::collections::VecDeque;

use crate::{commons::{damage_model::DamageComponent, resource_map::ResourceMap, rng::Rng}, engine::{animation::Animation, asset::{image::ImageAsset, image_sheet::ImageSheetAsset}, audio::SoundEffect, geometry::{Coord2, Size2D}, scene::Update, Palette}, game::{actor::{actor::ActorType, damage_resolver::{resolve_damage, DamageOutput}, health_component::BodyPart}, chunk::{Chunk, ChunkMap, TileMetadata}, effect_layer::EffectLayer, game_log::{GameLog, GameLogEntry, GameLogPart}}, world::{item::ItemId, world::World}, Actor, EquipmentType, GameContext, GameSceneState};

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
    Spell {
        target: SpellTarget,
        area: SpellArea,
        effects: Vec<SpellEffect>,
        // Effects
        cast: Option<(ImageSheetAsset, f32)>,
        projectile: Option<SpellProjectile>,
        // TODO: Struct
        impact: Option<(ImageSheetAsset, f32, ImpactPosition, bool)>,
        impact_sound: Option<SoundEffect>,
    },
    Targeted {
        damage: Option<DamageType>,
        inflicts: Option<Infliction>
    },
    Dig,
    PickUp,
    Sleep
}

#[derive(Clone)]
pub(crate) enum ImpactPosition {
    Cursor,
    EachTarget,
    EachTile,
}

#[derive(Clone)]
pub(crate) enum SpellTarget {
    /// Action is cast at the casters location
    Caster,
    /// Action is targeted at a actors location
    Actor { range: u16 },
    /// Any tile
    Tile { range: u16 },
}

#[derive(Clone)]
pub(crate) enum SpellArea {
    /// Affects only the targeted tile
    Target,
    /// Affects in an circle area
    Circle { radius: f32 },
    /// Affects in an rectangular area
    Rectangle(Size2D),
}

#[derive(Clone)]
pub(crate) struct SpellProjectile {
    pub(crate) position: ImpactPosition,
    pub(crate) wait: bool,
    pub(crate) projectile_type: SpellProjectileType
}

#[derive(Clone)]
pub(crate) enum SpellProjectileType {
    Projectile { sprite: ImageSheetAsset, speed: f32 }
}


impl SpellArea {

    pub(crate) fn bounding_box(&self, center: Coord2) -> (Coord2, Coord2) {
        match self {
            SpellArea::Target => (center, center),
            SpellArea::Circle { radius } => {
                let radius = *radius as i32;
                let start = center - Coord2::xy(radius, radius);
                let end = center + Coord2::xy(radius, radius);
                (start, end)
            },
            SpellArea::Rectangle(size) => {
                let start = center - Coord2::xy(size.x() as i32 / 2, size.y() as i32 / 2);
                let end = center + Coord2::xy(size.x() as i32 / 2, size.y() as i32 / 2);
                (start, end)
            }
        }
    }

    pub(crate) fn points(&self, center: Coord2) -> Vec<Coord2> {
        let mut vec = Vec::new();
        let (start, end) = self.bounding_box(center);
        for x in start.x..end.x+1 {
            for y in start.y..end.y+1 {
                let point = Coord2::xy(x, y);
                if self.point_in_area(center, point) {
                    vec.push(point);
                }
            }
        }
        return vec;
    }

    pub(crate) fn point_in_area(&self, center: Coord2, point: Coord2) -> bool {
        match self {
            SpellArea::Target => point == center,
            SpellArea::Circle { radius } => {
                let radius = (radius * radius) as f32;
                return point.dist_squared(&center) <= radius
            },
            SpellArea::Rectangle(size) => {
                let start = center - Coord2::xy(size.x() as i32 / 2, size.y() as i32 / 2);
                let end = center + Coord2::xy(size.x() as i32 / 2, size.y() as i32 / 2);
                return point.x >= start.x && point.y >= start.y && point.x <= end.x && point.y <= end.y;
            }
        }
    }

    pub(crate) fn filter<'a, A>(&self, center: Coord2, actor_index: usize, iter: impl Iterator<Item = A> + 'a) -> Box<dyn Iterator<Item = (usize, A)> + 'a> where A: std::borrow::Borrow<Actor> + 'a, {
        match self {
            SpellArea::Target => {
                return Box::new(iter.enumerate().filter(move |(_idx, actor): &(usize, A)| {
                    return actor.borrow().xy == center
                }));
            },
            SpellArea::Circle { radius } => {
                let radius = (radius * radius) as f32;
                return Box::new(iter.enumerate().filter(move |(idx, actor): &(usize, A)| {
                    if *idx == actor_index {
                        return false;
                    }
                    return actor.borrow().xy.dist_squared(&center) < radius
                }));
            },
            SpellArea::Rectangle(size) => {
                let start = center - Coord2::xy(size.x() as i32 / 2, size.y() as i32 / 2);
                let end = center + Coord2::xy(size.x() as i32 / 2, size.y() as i32 / 2);
                return Box::new(iter.enumerate().filter(move |(idx, actor): &(usize, A)| {
                    if *idx == actor_index {
                        return false;
                    }
                    let pos = actor.borrow().xy;
                    return pos.x >= start.x && pos.y >= start.y && pos.x <= end.x && pos.y <= end.y;
                }));
            }
        }
    }
    
}

#[derive(Clone)]
pub(crate) enum SpellEffect {
    /// Damages the target
    Damage(DamageComponent),
    /// Inflicts an effect on the target
    Inflicts { affliction: Affliction },
    /// Replaces tiles in the object layer
    ReplaceObject { tile: usize },
    /// Teleport the actor to the target
    TeleportActor,
    /// Inspects the target
    Inspect
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
    OnFire { duration: usize },
    Stunned { duration: usize }
}

impl Affliction {
    pub(crate) fn name_color(&self) -> (&str, Palette) {
        match self {
            Affliction::Bleeding { duration: _ } => ("Bleeding", Palette::Red),
            Affliction::Poisoned { duration: _ } => ("Poisoned", Palette::Green),
            Affliction::OnFire { duration: _ } => ("On Fire", Palette::Red),
            Affliction::Stunned { duration: _ } => ("Stunned", Palette::Gray),
        }
    }
}

pub(crate) struct ActionRunner {
    running_action: Option<RunningAction>
}

impl ActionRunner {

    pub(crate) fn new() -> Self {
        return Self {
            running_action: None
        }
    }

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

    pub(crate) fn can_use(action: &Action, actor_index: usize, cursor: Coord2, chunk: &mut Chunk) -> Result<(), ActionFailReason> {
        let actor = chunk.actor_mut(actor_index).unwrap();
        if !actor.ap.can_use(action.ap_cost) {
            return Err(ActionFailReason::NotEnoughAP);
        }
        if !actor.stamina.can_use(action.stamina_cost) {
            return Err(ActionFailReason::NotEnoughStamina);
        }
        match &action.action_type {
            ActionType::Targeted { damage: _, inflicts: _ } => {
                if actor.xy.dist_squared(&cursor) >= 3. {
                    return Err(ActionFailReason::CantReach);
                }
                let target = chunk.actors.iter_mut().enumerate().find(|(_, npc)| npc.xy == cursor);
                if target.is_none() {
                    return Err(ActionFailReason::NoValidTarget);
                }
            },
            ActionType::Spell { target, area: _, effects: _, cast: _, projectile: _, impact: _, impact_sound: _ } => {
                match target {
                    SpellTarget::Caster => return Ok(()),
                    SpellTarget::Actor { range } => {
                        if actor.xy.dist_squared(&cursor) >= (range*range) as f32 {
                            return Err(ActionFailReason::CantReach);
                        }
                        let target = chunk.actors.iter_mut().enumerate().find(|(_, npc)| npc.xy == cursor);
                        if target.is_none() {
                            return Err(ActionFailReason::NoValidTarget);
                        }
                    },
                    SpellTarget::Tile { range } => {
                        if actor.xy.dist_squared(&cursor) >= (range*range) as f32 {
                            return Err(ActionFailReason::CantReach);
                        }
                    }
                }
            },
            ActionType::PickUp => {
                if actor.xy.dist_squared(&cursor) >= 3. {
                    return Err(ActionFailReason::CantReach);
                }
                let item_on_ground = chunk.map.items_on_ground.iter().enumerate().find(|(_, (xy, _item, _tex))| *xy == cursor);
                if item_on_ground.is_none() {
                    return Err(ActionFailReason::NoValidTarget);
                }
            },
            ActionType::Dig => {
                if actor.xy.dist_squared(&cursor) >= 3. {
                    return Err(ActionFailReason::CantReach);
                }
                let tile_metadata = chunk.map.tiles_metadata.get(&cursor).and_then(|m| Some(m));
                if tile_metadata.is_none() {
                    return Err(ActionFailReason::NoValidTarget);
                }
            },
            ActionType::Sleep =>  {
                if actor.xy.dist_squared(&cursor) >= 3. {
                    return Err(ActionFailReason::CantReach);
                }
                // TODO: Bed
                let object_tile = chunk.map.get_object_idx(cursor);
                if object_tile != 3 {
                    return Err(ActionFailReason::NoValidTarget);
                }
            },
            // TODO:
            ActionType::Move { offset:_ } => (),
        }
        return Ok(());
    }

    pub(crate) fn try_use(&mut self, action: &Action, actor_index: usize, cursor: Coord2, chunk: &mut Chunk, world: &mut World, effect_layer: &mut EffectLayer, game_log: &mut GameLog, ctx: &GameContext) -> Result<Vec<ActionSideEffect>, ActionFailReason> {
        let r = Self::can_use(action, actor_index, cursor, chunk);
        if let Err(reason) = r {
            return Err(reason);
        }
        drop(r);

        let mut target = chunk.actors.iter_mut().enumerate().find(|(_, npc)| npc.xy == cursor);
        let mut item_on_ground = chunk.map.items_on_ground.iter().enumerate().find(|(_, (xy, _item, _tex))| *xy == cursor);
        let mut tile_metadata = chunk.map.tiles_metadata.get(&cursor).and_then(|m| Some(m));

        match &action.action_type {
            ActionType::Targeted { damage: _, inflicts: _ } => {
                if let Some((i, target)) = &mut target {
                    // TODO(QZ94ei4M): Borrow issues
                    // let mut actor = chunk.actor_mut(actor_index).unwrap();
                    let mut actor = &mut chunk.player;
                    if ActionRunner::targeted_try_use(action, &mut actor, target, effect_layer, game_log, &world, ctx) {
                        let mut side_effects = Vec::new();
                        if target.hp.health_points() == 0. {
                            actor.add_xp(100);
                            side_effects.push(ActionSideEffect::RemoveNpc(*i));
                        }
                        if target.actor_type != ActorType::Player {
                            side_effects.push(ActionSideEffect::MakeNpcsHostile);
                        }
                        return Ok(side_effects);
                    }
                }
            },
            ActionType::Spell { target, area, effects, cast, projectile, impact, impact_sound } => {
                let actor = chunk.actor_mut(actor_index).unwrap();

                // actor.ap.consume(action.ap_cost);
                // actor.stamina.consume(action.stamina_cost);

                game_log.log(GameLogEntry::from_parts(vec!(
                    GameLogPart::Actor(GameLogEntry::actor_name(actor, world, &ctx.resources), actor.actor_type),
                    GameLogPart::Text(format!(" used {}", action.name))
                )));

                let pos = match target {
                    SpellTarget::Caster => actor.xy.clone(),
                    SpellTarget::Actor { range: _ } => cursor,
                    SpellTarget::Tile { range: _ } => cursor,
                };

                // let target_actors = area
                //     .filter(pos, actor_index, chunk.actors_iter_mut())
                //     .map(|(i, _actor)| i)
                //     .collect();

                let mut steps = VecDeque::new();
                if let Some(cast) = cast {
                    // TODO: Wait
                    // TODO: Position
                    steps.push_back(RunningActionStep::CastSprite(cast.0.clone(), cast.1 as f64));
                }

                if let Some(projectile) = projectile {

                    // TODO: Dupped code
                    match projectile.position {
                        ImpactPosition::Cursor => steps.push_back(RunningActionStep::Projectile(projectile.clone(), pos)),
                        ImpactPosition::EachTarget => {
                            // TODO: Dupped code
                            let target_actors: Vec<usize> = area
                                .filter(pos, actor_index, chunk.actors_iter_mut())
                                .map(|(i, _actor)| i)
                                .collect();
                            for i in target_actors.iter() {
                                let actor = chunk.actor(*i).unwrap();
                                steps.push_back(RunningActionStep::Projectile(projectile.clone(), actor.xy.clone()))
                            }
                        }
                        ImpactPosition::EachTile => {
                            for p in area.points(pos) {
                                steps.push_back(RunningActionStep::Projectile(projectile.clone(), p))
                            }
                        }
                    }

                    if projectile.wait {
                        match projectile.projectile_type {
                            // TODO: Compute wait
                            SpellProjectileType::Projectile { sprite: _, speed } => steps.push_back(RunningActionStep::Wait(0.2))
                        }
                        
                    }
                }

                if let Some(impact_sound) = impact_sound {
                    steps.push_back(RunningActionStep::Sound(impact_sound.clone()));
                }
                if let Some(impact) = impact {
                    // TODO: Dupped code
                    match impact.2 {
                        ImpactPosition::Cursor => steps.push_back(RunningActionStep::Sprite(impact.0.clone(), pos)),
                        ImpactPosition::EachTarget => {
                            // TODO: Dupped code
                            let target_actors: Vec<usize> = area
                                .filter(pos, actor_index, chunk.actors_iter_mut())
                                .map(|(i, _actor)| i)
                                .collect();
                            for i in target_actors.iter() {
                                let actor = chunk.actor(*i).unwrap();
                                steps.push_back(RunningActionStep::Sprite(impact.0.clone(), actor.xy.clone()))
                            }
                        }
                        ImpactPosition::EachTile => {
                            for p in area.points(pos) {
                                steps.push_back(RunningActionStep::Sprite(impact.0.clone(), p))
                            }
                        }
                    }
                    if impact.3 {
                        // TODO: Compute duration
                        steps.push_back(RunningActionStep::Wait(impact.1 as f64));
                    }
                }

                steps.push_back(RunningActionStep::Effect(effects.clone()));


                self.running_action = Some(RunningAction {
                    actor: actor_index,
                    spell_area: area.clone(),
                    center: pos,
                    // target_actors,
                    current_step: None,
                    steps
                });

                // TODO(w0ScmN4f): Move down
                if let Some(fx) = &action.sound_effect {
                    ctx.audio.play_once(fx.clone());
                }
            },
            ActionType::PickUp => {
                let (i, (_, item, _)) = item_on_ground.as_mut().expect("msg");
                // TODO(QZ94ei4M): Borrow issues
                // let mut actor = chunk.actor_mut(actor_index).unwrap();
                let actor = &mut chunk.player;
                if let Ok(_) = actor.inventory.add(item.clone()) {
                    return Ok(vec!(ActionSideEffect::RemoveItemOnGround(*i)))
                }
            },
            ActionType::Dig => {
                if let Some(meta) = &mut tile_metadata {
                    match meta {
                        TileMetadata::BurialPlace(creature_id) => {
                            let mut side_effects = Vec::new();
                            side_effects.push(ActionSideEffect::RemoveObject(cursor));
                            let creature = world.creatures.get(creature_id);
                            if let Some(details) = &creature.details {
                                for item in details.inventory.iter() {
                                    side_effects.push(ActionSideEffect::AddArtifactOnGround(cursor, *item));
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

    pub(crate) fn update(&mut self, update: &Update, chunk: &mut Chunk, world: &mut World, effect_layer: &mut EffectLayer, game_log: &mut GameLog, ctx: &GameContext) {
        let mut clear_running_action = false;
        if let Some(action) = &mut self.running_action {

            if action.current_step.is_none() {
                let step = action.steps.pop_front();
                if let Some(step) = step {
                    let duration = step.duration();


                    match &step {
                        RunningActionStep::Effect(effects) => {
                            // for i in action.target_actors.iter() {
                            //     let target = chunk.actor_mut(*i).unwrap();

                                for effect in effects.iter() {
                                    match effect {
                                        SpellEffect::Damage(model) => {

                                            // TODO: Dupped code
                                            let target_actors: Vec<usize> = action.spell_area
                                                .filter(action.center, action.actor, chunk.actors_iter_mut())
                                                .map(|(i, _actor)| i)
                                                .collect();
                                            for i in target_actors.iter() {
                                                let target = chunk.actor_mut(*i).unwrap();
                                                
                                                let target_body_part = BodyPart::random(&mut Rng::rand());
                                                let damage = resolve_damage(&model, &target.stats(), &target_body_part, &target.stats());
                    
                                                match damage {
                                                    DamageOutput::Dodged => {
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
                                                game_log.log(GameLogEntry::damage(target, &damage, &world, &ctx.resources));
                    
                                                // if let Some(fx) = &action.sound_effect {
                                                //     ctx.audio.play_once(fx.clone());
                                                // }
                                                // Animations
                                                let dir = target.xy - target.xy;
                                                target.animation.play(&Self::build_attack_anim(dir));
                                                target.animation.play(&&Self::build_hurt_anim(dir));

                                                let dead = target.hp.health_points();
                                                let actor_type = target.actor_type;

                                                if dead == 0. {
                                                    let actor = chunk.actor_mut(action.actor).unwrap();
                                                    actor.add_xp(100);
                                                    chunk.remove_npc(*i, ctx);
                                                }
                                                if actor_type != ActorType::Player {
                                                    for p in chunk.actors_iter_mut() {
                                                        if p.actor_type != ActorType::Player {
                                                            p.actor_type = ActorType::Hostile;
                                                        }
                                                    }
                                                }
                                            }

                                        },
                                        SpellEffect::Inflicts { affliction } => {
                                            // TODO: Dupped code
                                            let target_actors: Vec<usize> = action.spell_area
                                                .filter(action.center, action.actor, chunk.actors_iter_mut())
                                                .map(|(i, _actor)| i)
                                                .collect();
                                            for i in target_actors.iter() {
                                                let target = chunk.actor_mut(*i).unwrap();


                                                let (name, color) = affliction.name_color();
                                                game_log.log(GameLogEntry::from_parts(vec!(
                                                    GameLogPart::Actor(GameLogEntry::actor_name(target, world, &ctx.resources), target.actor_type),
                                                    GameLogPart::Text(format!(" is {}", name))
                                                )));
                                                effect_layer.add_text_indicator(target.xy, name, color);
                                                target.add_affliction(&affliction)
                                            }
                                        },
                                        SpellEffect::ReplaceObject { tile } => {
                                            // TODO: Bounding box
                                            for x in 0..chunk.size.0 {
                                                for y in 0..chunk.size.1 {
                                                    if action.spell_area.point_in_area(action.center, Coord2::xy(x as i32, y as i32)) {
                                                        chunk.map.object_layer.set_tile(x, y, *tile);
                                                    }
                                                }
                                            }
                                        },
                                        SpellEffect::TeleportActor => {
                                            let actor = chunk.actor_mut(action.actor).unwrap();
                                            actor.xy = action.center
                                        },
                                        SpellEffect::Inspect => {

                                            // TODO(hu2htwck): Add info to codex

                                            println!("Inspect at {:?}", action.center);

                                            // TODO: Dupped code
                                            let target_actors: Vec<usize> = action.spell_area
                                                .filter(action.center, action.actor, chunk.actors_iter_mut())
                                                .map(|(i, _actor)| i)
                                                .collect();
                                            for i in target_actors.iter() {
                                                let target = chunk.actor_mut(*i).unwrap();

                                                let creature_id = target.creature_id;
                                                if let Some(creature_id) = creature_id {
                                                    let codex = world.codex.creature_mut(&creature_id);
                                                    // TODO(hu2htwck): Not this
                                                    codex.add_appearance();
                                                    codex.add_name();
                                                    let creature = world.creatures.get(&creature_id);
                                                    println!("Target: {}, {:?}, {:?} birth {}", creature.name(&creature_id, &world, &ctx.resources), creature.profession, creature.gender, creature.birth.year());
                                                    // TODO(IhlgIYVA): Debug print
                                                    println!("Relationships: {:?}", creature.relationships)

                                                }
                                            }
                                            // if let Some((_, (_, item, _))) = &item_on_ground {
                                            //     println!("{}", item.description(&ctx.resources, &world));
                                            // }
                                            // let tile = chunk.map.get_object_idx(cursor);
                                            // let tile_meta = &tile_metadata;
                                            // match tile {
                                            //     1 => println!("A wall."),
                                            //     2 => println!("A tree."),
                                            //     3 => println!("A bed."),
                                            //     4 => println!("A table."),
                                            //     5 => println!("A stool."),
                                            //     6 => println!("A tombstone."),            
                                            //     _ => ()                                
                                            // };

                                            // if let Some(meta) = tile_meta {
                                            //     match meta {
                                            //         TileMetadata::BurialPlace(creature_id) => {
                                            //             let creature = world.creatures.get(creature_id);
                                            //             if let Some(death) = creature.death {
                                            //                 let codex = world.codex.creature_mut(&creature_id);
                                            //                 codex.add_name();
                                            //                 codex.add_death();
                                            //                 // TODO(hu2htwck): Event
                                            //                 println!("The headstone says: \"Resting place of {:?}\". {} - {}. Died from {:?}", creature_id, creature.birth.year(), death.0.year(), death.1);
                                            //             }
                                                        
                                            //         }
                                            //     }
                                            // }
                                        }
                                    }
                                }
                            // }
                        },
                        RunningActionStep::Wait(_) => {}
                        RunningActionStep::Projectile(projectile, to) => {
                            let actor = chunk.actor(action.actor).unwrap();
                            match &projectile.projectile_type {
                                SpellProjectileType::Projectile { sprite, speed } => {
                                    effect_layer.add_projectile(actor.xy, *to, *speed as f64, sprite.clone());
                                }
                            }
                        },
                        // TODO(w0ScmN4f):: Naming doesn't make sense
                        RunningActionStep::Sprite(sprite, pos) => {
                            effect_layer.play_sprite(*pos, sprite.clone());
                        },
                        RunningActionStep::CastSprite(sprite, _) => {
                            let actor = chunk.actor(action.actor).unwrap();
                            effect_layer.play_sprite(actor.xy, sprite.clone());
                        },
                        RunningActionStep::Sound(sound) => {
                            ctx.audio.play_once(sound.clone());
                        }
                    }


                    action.current_step = Some((step, 0., duration)); 


                } else {
                    action.current_step = None; 
                }
            }

            if action.current_step.is_none() {
                clear_running_action = true;
            }

            if let Some(step) = &mut action.current_step {
                step.1 += update.delta_time;

                if step.1 > step.2 {
                    action.current_step = None;
                }
            }

        }
        if clear_running_action {
            self.running_action = None;
        }
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

                            game_log.log(GameLogEntry::damage(target, &damage, &world, &ctx.resources));

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

struct RunningAction {
    actor: usize,
    center: Coord2,
    spell_area: SpellArea,
    // target_actors: Vec<usize>,
    current_step: Option<(RunningActionStep, f64, f64)>,
    steps: VecDeque<RunningActionStep>
}

enum RunningActionStep {
    Projectile(SpellProjectile, Coord2),
    Effect(Vec<SpellEffect>),
    // TODO: Sprite + Wait
    CastSprite(ImageSheetAsset, f64),
    Wait(f64),
    Sprite(ImageSheetAsset, Coord2),
    Sound(SoundEffect),
}

impl RunningActionStep {

    fn duration(&self) -> f64 {
        match self {
            Self::Projectile(_, _) => 0.,
            Self::Effect(_) => 0.,
            Self::Sprite(_, _) => 0.,
            Self::Wait(d) => *d,
            Self::CastSprite(_, d) => *d,
            Self::Sound(_) => 0.
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
            Self::RemoveNpc(i) => game.chunk.remove_npc(*i, ctx),
            Self::MakeNpcsHostile => {
                for p in game.chunk.actors.iter_mut() {
                    p.actor_type = ActorType::Hostile;
                }
            },
            Self::RemoveItemOnGround(i) => {
                game.chunk.map.items_on_ground.remove(*i);
            },
            Self::RemoveObject(pos) => {
                game.chunk.map.remove_object(*pos);
                game.chunk.map.tiles_metadata.remove(pos);
            },
            Self::AddArtifactOnGround(pos, item) => {
                let item = game.world.artifacts.get(item);
                game.chunk.map.items_on_ground.push((*pos, item.clone(), item.make_texture(&ctx.resources.materials)));
            }
        }
    }

}