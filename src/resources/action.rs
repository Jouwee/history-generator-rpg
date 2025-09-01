use std::collections::VecDeque;

use math::Vec2i;
use serde::{Deserialize, Serialize};

use crate::{commons::{bitmask::bitmask_get, damage_model::DamageRoll, id_vec::Id, interpolate::lerp, resource_map::ResourceMap, rng::Rng}, engine::{animation::Animation, assets::{assets, ImageSheetAsset}, audio::SoundEffect, geometry::Coord2, scene::{BusEvent, ShowChatDialogData, ShowInspectDialogData, Update}, Palette}, game::{actor::{damage_resolver::{resolve_damage, DamageOutput}, health_component::BodyPart}, chunk::TileMetadata, effect_layer::EffectLayer, game_log::{GameLog, GameLogEntry, GameLogPart}, inventory::inventory::EquipmentType, state::{GameState, PLAYER_IDX}}, resources::{item_blueprint::ItemMaker, object_tile::ObjectTileId, resources::resources}, world::{date::Duration, world::World}, Actor, GameContext, SPRITE_FPS};

// TODO(0xtBbih5): Should serialize the string id, not the internal id
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
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
    pub(crate) icon: String,
    pub(crate) description: String,
    pub(crate) ap_cost: u16,
    pub(crate) stamina_cost: f32,
    pub(crate) cooldown: u16,
    pub(crate) log_use: bool,
    pub(crate) target: ActionTarget,
    pub(crate) area: ActionArea,
    pub(crate) effects: Vec<ActionEffect>,
    // Effects
    pub(crate) cast_sprite: Option<(ImageSheetAsset, f32)>,
    pub(crate) cast_sfx: Option<SoundEffect>,
    pub(crate) projectile: Option<ActionProjectile>,
    pub(crate) impact_sprite: Option<(ImageSheetAsset, ImpactPosition, bool)>,
    pub(crate) impact_sfx: Option<SoundEffect>,
    pub(crate) damage_sfx: Option<SoundEffect>,
}

#[derive(Clone)]
pub(crate) enum ImpactPosition {
    Cursor,
    EachTarget,
    EachTile,
}

pub(crate) const FILTER_CAN_OCCUPY: u8 = 0b0000_0001;
pub(crate) const FILTER_CAN_VIEW: u8 = 0b0000_0010;
pub(crate) const FILTER_CAN_DIG: u8 = 0b0000_0100;
pub(crate) const FILTER_CAN_SLEEP: u8 = 0b0000_1000;
pub(crate) const FILTER_ITEM: u8 = 0b0001_0000;
pub(crate) const FILTER_NOT_HOSTILE: u8 = 0b0010_0000;
pub(crate) const FILTER_CAN_HARVEST: u8 = 0b0100_0000;

#[derive(Clone)]
pub(crate) enum ActionTarget {
    /// Action is cast at the casters location
    Caster,
    /// Action is targeted at a actors location
    Actor { range: f32, filter_mask: u8 },
    /// Any tile
    Tile { range: f32, filter_mask: u8 },
}

impl ActionTarget {
    
    pub(crate) fn can_use(&self, actor_pos: &Coord2, chunk: &GameState, cursor: &Coord2) -> Result<(), ActionFailReason> {
        match &self {
            ActionTarget::Caster => return Ok(()),
            ActionTarget::Actor { range, filter_mask } => {
                if actor_pos.dist_squared(&cursor) > (range*range) as f32 {
                    return Err(ActionFailReason::CantReach);
                }
                if !chunk.chunk.size.in_bounds(*cursor) {
                    return Err(ActionFailReason::CantReach);
                }
                if bitmask_get(*filter_mask, FILTER_CAN_VIEW) {
                    if !chunk.chunk.check_line_of_sight(&actor_pos, &cursor) {
                        return Err(ActionFailReason::CantSee);
                    }
                }
                let target = chunk.actors_iter().find(|npc| npc.xy == cursor.to_vec2i());
                let target = target.ok_or_else(|| ActionFailReason::NoValidTarget)?;

                // SMELL: Maybe not the best way to check self?
                if actor_pos == cursor {
                    return Err(ActionFailReason::NoValidTarget);
                }

                if bitmask_get(*filter_mask, FILTER_NOT_HOSTILE) {
                    // TODO: Maybe not player
                    if chunk.ai_groups.is_hostile(target.ai_group, chunk.player().ai_group) {
                        return Err(ActionFailReason::NoValidTarget);
                    }
                }
            },
            ActionTarget::Tile { range, filter_mask } => {
                if actor_pos.dist_squared(&cursor) > (range*range) as f32 {
                    return Err(ActionFailReason::CantReach);
                }
                if !chunk.chunk.size.in_bounds(*cursor) {
                    return Err(ActionFailReason::CantReach);
                }
                if bitmask_get(*filter_mask, FILTER_CAN_OCCUPY) {
                    if !chunk.can_occupy(cursor) {
                        return Err(ActionFailReason::NoValidTarget);
                    }
                }
                if bitmask_get(*filter_mask, FILTER_CAN_DIG) {
                    let tile_metadata = chunk.chunk.tiles_metadata.get(&cursor.to_vec2i()).and_then(|m| Some(m));
                    if tile_metadata.is_none() {
                        return Err(ActionFailReason::NoValidTarget);
                    }
                }
                if bitmask_get(*filter_mask, FILTER_CAN_SLEEP) {
                    let object_tile = chunk.chunk.get_object_id(*cursor);
                    let resources = resources();
                    if let Some(object_tile) = object_tile {
                        if object_tile != resources.object_tiles.id_of("obj:bed") {
                            return Err(ActionFailReason::NoValidTarget);
                        }
                    } else {
                        return Err(ActionFailReason::NoValidTarget);   
                    }
                }

                if bitmask_get(*filter_mask, FILTER_CAN_HARVEST) {
                    let resources = resources();
                    let object_tile = chunk.chunk.get_object_id(*cursor);
                    if let Some(object_tile) = object_tile {
                        let object_tile = resources.object_tiles.get(&object_tile);
                        if object_tile.harvestable.is_none() {
                            return Err(ActionFailReason::NoValidTarget);
                        }
                    } else {
                        return Err(ActionFailReason::NoValidTarget);   
                    }
                }

                if bitmask_get(*filter_mask, FILTER_ITEM) {
                    let item_on_ground = chunk.chunk.items_on_ground.iter().enumerate().find(|(_, (xy, _item, _tex))| xy == cursor);
                    if item_on_ground.is_none() {
                        return Err(ActionFailReason::NoValidTarget);
                    }
                }
                if bitmask_get(*filter_mask, FILTER_CAN_VIEW) {
                    if !chunk.chunk.check_line_of_sight(actor_pos, &cursor) {
                        return Err(ActionFailReason::NoValidTarget);
                    }
                }
            }
        }
        return Ok(())
    }

}

#[derive(Clone, PartialEq)]
pub(crate) enum ActionArea {
    /// Affects only the targeted tile
    Target,
    /// Affects in an circle area
    Circle { radius: f32 },
}

#[derive(Clone)]
pub(crate) struct ActionProjectile {
    pub(crate) position: ImpactPosition,
    pub(crate) wait: bool,
    pub(crate) projectile_type: SpellProjectileType
}

#[derive(Clone)]
pub(crate) enum SpellProjectileType {
    Projectile { sprite: ImageSheetAsset, speed: f32 }
}


impl ActionArea {

    pub(crate) fn bounding_box(&self, center: Vec2i) -> (Vec2i, Vec2i) {
        match self {
            ActionArea::Target => (center, center),
            ActionArea::Circle { radius } => {
                let radius = *radius as i32;
                let start = center - Vec2i(radius, radius);
                let end = center + Vec2i(radius, radius);
                (start, end)
            }
        }
    }

    pub(crate) fn points(&self, center: Coord2) -> Vec<Coord2> {
        let mut vec = Vec::new();
        let (start, end) = self.bounding_box(center.to_vec2i());
        for x in start.x()..end.x()+1 {
            for y in start.y()..end.y()+1 {
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
            ActionArea::Target => point == center,
            ActionArea::Circle { radius } => {
                let radius = (radius * radius) as f32;
                return point.dist_squared(&center) <= radius
            }
        }
    }

    pub(crate) fn actors_indices<'a, A>(&self, center: Coord2, actor_index: usize, iter: impl Iterator<Item = A> + 'a) -> Vec<usize> where A: std::borrow::Borrow<Actor> + 'a {
        return self
            .filter(center, actor_index, iter)
            .map(|(i, _actor)| i)
            .collect();
    }

    pub(crate) fn filter<'a, A>(&self, center: Coord2, actor_index: usize, iter: impl Iterator<Item = A> + 'a) -> Box<dyn Iterator<Item = (usize, A)> + 'a> where A: std::borrow::Borrow<Actor> + 'a {
        match self {
            ActionArea::Target => {
                return Box::new(iter.enumerate().filter(move |(idx, actor): &(usize, A)| {
                    if *idx == actor_index {
                        return false;
                    }
                    return actor.borrow().xy == center.to_vec2i()
                }));
            },
            ActionArea::Circle { radius } => {
                let radius = (radius * radius) as f32;
                return Box::new(iter.enumerate().filter(move |(idx, actor): &(usize, A)| {
                    if *idx == actor_index {
                        return false;
                    }
                    return actor.borrow().xy.dist_squared(&center.to_vec2i()) < radius
                }));
            }
        }
    }
    
}

#[derive(Debug, Clone)]
pub(crate) enum ActionEffect {
    /// Damages the target
    Damage { add_weapon: bool, damage: DamageRoll },
    /// Inflicts an effect on the target
    Inflicts { affliction: Affliction },
    /// Replaces tiles in the object layer
    ReplaceObject { tile: ObjectTileId },
    /// Teleport the actor to the target
    TeleportActor,
    /// Walk the actor to the target
    Walk,
    /// Inspects the target
    Inspect,
    /// Talks with the target
    Talk,
    /// Digs graves
    Dig,
    /// Sleeps
    Sleep,
    /// Harvest something from the object
    Harvest,
    /// PickUp items
    PickUp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Affliction {
    Bleeding { duration: usize },
    Poisoned { duration: usize },
    OnFire { duration: usize },
    Stunned { duration: usize },
    Healing { duration: usize, strength: f32 },
    Recovery { duration: usize, strength: f32 },
}

impl Affliction {
    pub(crate) fn name_color(&self) -> (&str, Palette) {
        match self {
            Affliction::Bleeding { duration: _ } => ("Bleeding", Palette::Red),
            Affliction::Poisoned { duration: _ } => ("Poisoned", Palette::Green),
            Affliction::OnFire { duration: _ } => ("On Fire", Palette::Red),
            Affliction::Stunned { duration: _ } => ("Stunned", Palette::Gray),
            Affliction::Healing { duration: _, strength : _ } => ("Healing", Palette::Red),
            Affliction::Recovery { duration: _, strength : _ } => ("Recovery", Palette::Red),
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

    pub(crate) fn can_use(action_id: &ActionId, action: &Action, actor_index: usize, cursor: Coord2, chunk: &GameState) -> Result<(), ActionFailReason> {
        let actor = chunk.actor(actor_index).unwrap();
        if !actor.ap.can_use(action.ap_cost) {
            return Err(ActionFailReason::NotEnoughAP);
        }
        if !actor.stamina.can_use(action.stamina_cost) {
            return Err(ActionFailReason::NotEnoughStamina);
        }
        if actor.cooldowns.iter().any(|cooldown| cooldown.0 == *action_id) {
            return Err(ActionFailReason::OnCooldown);
        }
        return action.target.can_use(&actor.xy.into(), chunk, &cursor);
    }

    pub(crate) fn try_use(&mut self, action_id: &ActionId, action: &Action, actor_index: usize, cursor: Coord2, chunk: &mut GameState, world: &mut World, game_log: &mut GameLog, ctx: &GameContext) -> Result<(), ActionFailReason> {
        let r = Self::can_use(action_id, action, actor_index, cursor, chunk);
        if let Err(reason) = r {
            return Err(reason);
        }
        drop(r);

        let actor = chunk.actor_mut(actor_index).unwrap();

        actor.ap.consume(action.ap_cost);
        actor.stamina.consume(action.stamina_cost);
        if action.cooldown > 0 {
            actor.cooldowns.push((*action_id, action.cooldown));
        }

        if action.log_use {
            game_log.log(GameLogEntry::from_parts(vec!(
                GameLogPart::Actor(GameLogEntry::actor_name(actor, world, &ctx.resources), actor_index == PLAYER_IDX),
                GameLogPart::Text(format!(" used {}", action.name))
            )));
        }

        let pos = match &action.target {
            ActionTarget::Caster => actor.xy.clone().into(),
            ActionTarget::Actor { range: _, filter_mask: _ } => cursor,
            ActionTarget::Tile { range: _, filter_mask: _ } => cursor,
        };

        // Change facing accordingly
        let actor_xy: Coord2 = actor.xy.into();
        let angle_degrees = actor_xy.angle_between_deg(&pos);
        if angle_degrees >= 135. && angle_degrees <= 225. { // Facing mostly left
            actor.sprite_flipped = true;
        } else if angle_degrees >= 315. || angle_degrees <= 45. { // Facing mostly right
            actor.sprite_flipped = false;
        } // Else, leave as is

        let mut steps = VecDeque::new();

        if let Some(fx) = &action.cast_sfx {
            steps.push_back(RunningActionStep::Sound(fx.clone()));
        }

        if let Some(cast) = &action.cast_sprite {
            steps.push_back(RunningActionStep::Sprite(cast.0.clone(), actor.xy.into()));
            let sheet = assets().image_sheet(&cast.0.path, cast.0.tile_size.clone());
            steps.push_back(RunningActionStep::Wait(sheet.len() as f64 * SPRITE_FPS))
        }

        let impact_points = |position| {
            match &position {
                ImpactPosition::Cursor => vec!(pos),
                ImpactPosition::EachTarget => {
                    action.area.actors_indices(pos, actor_index, chunk.actors_iter()).iter().map(|i| {
                        let actor = chunk.actor(*i).unwrap();
                        actor.xy.into()
                    }).collect()
                }
                ImpactPosition::EachTile => action.area.points(pos)
            }
        };


        if let Some(projectile) = &action.projectile {

            let actor = chunk.actor(actor_index).unwrap();
            let from = actor.xy.clone();

            let mut longest_distance: f32 = 0.;

            for point in impact_points(projectile.position.clone()) {
                steps.push_back(RunningActionStep::Projectile(projectile.clone(), point));
                longest_distance = longest_distance.max(from.dist(&point.to_vec2i()));
            }

            if projectile.wait {
                match projectile.projectile_type {
                    SpellProjectileType::Projectile { sprite: _, speed } => {
                        let wait = longest_distance / speed;
                        steps.push_back(RunningActionStep::Wait(wait as f64))
                    }
                }
                
            }
        }

        if let Some(impact_sound) = &action.impact_sfx {
            steps.push_back(RunningActionStep::Sound(impact_sound.clone()));
        }
        if let Some(impact) = &action.impact_sprite {

            for point in impact_points(impact.1.clone()) {
                steps.push_back(RunningActionStep::Sprite(impact.0.clone(), point))
            }

            if impact.2 {
                let sheet = assets().image_sheet(&impact.0.path, impact.0.tile_size.clone());
                steps.push_back(RunningActionStep::Wait(sheet.len() as f64 * SPRITE_FPS))
            }
        }

        steps.push_back(RunningActionStep::Effect(action.effects.clone()));


        self.running_action = Some(RunningAction {
            action: action.clone(),
            actor: actor_index,
            spell_area: action.area.clone(),
            center: pos,
            current_step: None,
            steps
        });

        return Ok(());
    }

    pub(crate) fn update(&mut self, update: &Update, chunk: &mut GameState, world: &mut World, effect_layer: &mut EffectLayer, game_log: &mut GameLog, ctx: &mut GameContext) {
        let mut clear_running_action = false;
        if let Some(action) = &mut self.running_action {

            if action.current_step.is_none() {
                let step = action.steps.pop_front();
                if let Some(step) = step {
                    let mut duration = 0.;


                    match &step {
                        RunningActionStep::Effect(effects) => {
                            for effect in effects.iter() {
                                match effect {
                                    ActionEffect::Damage { damage, add_weapon } => {
                                        let actor = chunk.actor(action.actor).unwrap();
                                        let actor_xy = actor.xy.clone();
                                        let mut damage = damage.clone();

                                        if let Some(item) = actor.inventory.equipped(&EquipmentType::Hand) {
                                            if *add_weapon {
                                                damage = damage + item.total_damage(&ctx.resources.materials)
                                            } else {
                                                damage = damage + item.extra_damage(&ctx.resources.materials)
                                            }
                                        }

                                        for i in action.spell_area.actors_indices(action.center, action.actor, chunk.actors_iter_mut()) {
                                            let target_is_player = chunk.is_player(i);
                                            let target = chunk.actor_mut(i).unwrap();
                                            
                                            let target_body_part = BodyPart::random(&mut Rng::rand());

                                            let damage = resolve_damage(&damage, &target.stats(), &target_body_part, &target.stats());
                
                                            match damage {
                                                DamageOutput::Dodged => {
                                                    effect_layer.add_text_indicator(target.xy.into(), "Dodged", Palette::Gray);
                                                },
                                                DamageOutput::Hit(damage) => {
                                                    target.hp.hit(target_body_part, damage);
                                                    effect_layer.add_damage_number(target.xy.into(), damage);
                                                },
                                                DamageOutput::CriticalHit(damage) => {
                                                    target.hp.critical_hit(target_body_part, damage);
                                                    effect_layer.add_damage_number(target.xy.into(), damage);
                                                },
                                            }
                                            game_log.log(GameLogEntry::damage(target, target_is_player, &damage, &world, &ctx.resources));
                
                                            let dead = target.hp.health_points();
                                            let target_ai = target.ai_group;
                                            let xy = target.xy.clone();

                                            // Animations
                                            let dir = (xy - actor_xy).into();
                                            target.animation.play(&Self::build_hurt_anim(dir));

                                            // Hit sound effect
                                            if let Some(damage_sfx) = &action.action.damage_sfx {
                                                ctx.audio.play_once(damage_sfx.clone());
                                            }
                                            let species = ctx.resources.species.get(&target.species);
                                            if let Some(hurt_sfx) = &species.hurt_sound {
                                                ctx.audio.play_once(hurt_sfx.clone());
                                            }

                                            let actor = chunk.actor_mut(action.actor).unwrap();
                                            actor.animation.play(&Self::build_attack_anim(dir));
                                            let actor_ai = actor.ai_group;

                                            if dead == 0. {
                                                actor.add_xp(100);
                                                chunk.remove_npc(i, ctx);
                                            }

                                            if actor_ai != target_ai {
                                                chunk.ai_groups.make_hostile(actor_ai, target_ai);
                                            }
                                        }

                                    },
                                    ActionEffect::Inflicts { affliction } => {
                                        for i in action.spell_area.actors_indices(action.center, action.actor, chunk.actors_iter_mut()) {
                                            let target = chunk.actor_mut(i).unwrap();
                                            let (name, color) = affliction.name_color();
                                            game_log.log(GameLogEntry::from_parts(vec!(
                                                GameLogPart::Actor(GameLogEntry::actor_name(target, world, &ctx.resources), i == PLAYER_IDX),
                                                GameLogPart::Text(format!(" is {}", name))
                                            )));
                                            effect_layer.add_text_indicator(target.xy.into(), name, color);
                                            target.add_affliction(&affliction)
                                        }
                                    },
                                    ActionEffect::ReplaceObject { tile } => {
                                        for point in action.spell_area.points(action.center) {
                                            chunk.chunk.object_layer.set_tile(point.x as usize, point.y as usize, tile.as_usize() + 1);
                                        }
                                    },
                                    ActionEffect::TeleportActor => {
                                        let actor = chunk.actor_mut(action.actor).unwrap();
                                        actor.xy = action.center.to_vec2i()
                                    },
                                    ActionEffect::Walk => {
                                        let actor = chunk.actor_mut(action.actor).unwrap();
                                        let from = actor.xy.clone();
                                        actor.xy = action.center.to_vec2i();
                                        actor.animation.play(&Self::build_walk_anim(&from.into(), &action.center));
                                        if let Some(sound) = chunk.chunk.get_step_sound(action.center, &ctx.resources) {
                                            ctx.audio.play_once(sound);
                                        }
                                    },
                                    ActionEffect::Inspect => {
                                        let actor = action.spell_area.actors_indices(action.center, action.actor, chunk.actors_iter_mut())
                                            .first()
                                            .and_then(|i| chunk.actor(*i).cloned());

                                        let item = chunk.chunk.items_on_ground.iter().enumerate().find(|(_, (xy, _item, _tex))| *xy == action.center)
                                            .and_then(|(_, (_, item, _))| Some(item.clone()));

                                        let tile_metadata = chunk.chunk.tiles_metadata.get(&action.center.to_vec2i()).cloned();

                                        ctx.event_bus.push(BusEvent::ShowInspectDialog(ShowInspectDialogData {
                                            actor,
                                            item,
                                            tile_metadata,
                                        }))
                                    },
                                    ActionEffect::Talk => {
                                        let actor = action.spell_area.actors_indices(action.center, action.actor, chunk.actors_iter_mut())
                                            .first()
                                            .and_then(|i| chunk.actor(*i).cloned());
                                        if let Some(actor) = actor {
                                            ctx.event_bus.push(BusEvent::ShowChatDialog(ShowChatDialogData {
                                                world_coord: chunk.coord.xy.into(),
                                                actor,
                                            }))
                                        }
                                    },
                                    ActionEffect::Dig => {
                                        for point in action.spell_area.points(action.center) {
                                            let tile_metadata = chunk.chunk.tiles_metadata.get(&point.to_vec2i()).cloned();
                                            if let Some(meta) = &tile_metadata {
                                                match meta {
                                                    TileMetadata::BurialPlace(creature_id) => {

                                                        chunk.chunk.remove_object(point.clone());
                                                        chunk.chunk.tiles_metadata.remove(&point.to_vec2i());

                                                        let creature = world.creatures.get(creature_id);
                                                        if let Some(details) = &creature.details {
                                                            for item in details.inventory.iter() {
                                                                let item = world.artifacts.get(item);
                                                                chunk.chunk.items_on_ground.push((point, item.clone(), item.make_texture(&ctx.resources)));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    ActionEffect::Sleep =>  {
                                        let actor = chunk.actor_mut(action.actor).unwrap();
                                        actor.hp.recover_full();
                                        ctx.event_bus.push(BusEvent::SimulateTime(Duration::days(1)));
                                    },
                                    ActionEffect::Harvest => {
                                        let resources = resources();
                                        for point in action.spell_area.points(action.center) {
                                            // Gets the item
                                            let object = chunk.chunk.get_object_id(point).expect("Should be checked");
                                            let object = resources.object_tiles.get(&object);
                                            let harvestable = object.harvestable.as_ref().expect("Should be checked");
                                            let item_blueprint = resources.item_blueprints.get(&harvestable.drops);
                                            let item = item_blueprint.make(vec!(), &resources);
                                            // Adds to player inventory or drops on the ground
                                            let actor = chunk.actor_mut(action.actor).unwrap();
                                            let result = actor.inventory.add(item);
                                            if let Err(item) = result {
                                                let texture = item.make_texture(&resources);
                                                chunk.chunk.items_on_ground.push((action.center.clone(), item, texture));
                                            }
                                            // Removes the object
                                            chunk.chunk.object_layer.set_tile(point.x as usize, point.y as usize, 0);
                                        }
                                    }
                                    ActionEffect::PickUp => {
                                        let item_on_ground = match chunk.chunk.items_on_ground.iter().enumerate().find(|(_, (xy, _item, _tex))| *xy == action.center) {
                                            None => None,
                                            Some((i, (_, item, _))) => Some((i, item.clone()))
                                        };
                                        if let Some((i, item)) = item_on_ground {
                                            let actor = chunk.actor_mut(action.actor).unwrap();
                                            if let Ok(_) = actor.inventory.add(item.clone()) {
                                                chunk.chunk.items_on_ground.remove(i);
                                            }
                                        }
                                    },
                                }
                            }
                        },
                        RunningActionStep::Wait(d) => duration = *d,
                        RunningActionStep::Projectile(projectile, to) => {
                            let actor = chunk.actor(action.actor).unwrap();
                            match &projectile.projectile_type {
                                SpellProjectileType::Projectile { sprite, speed } => {
                                    effect_layer.add_projectile(actor.xy.into(), *to, *speed as f64, sprite.clone());
                                }
                            }
                        },
                        RunningActionStep::Sprite(sprite, pos) => {
                            effect_layer.play_sprite(*pos, sprite.clone());
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

    fn build_walk_anim(from: &Coord2, to: &Coord2) -> Animation {
        let start_offset = *from - *to;
        let start_offset = [start_offset.x as f64 * 24., start_offset.y as f64 * 24., 0.];
        let mid_point = [
            lerp(start_offset[0], 0., 0.5),
            lerp(start_offset[1], 0., 0.5),
            6.,
        ];
        Animation::new()
            .translate(0.08, start_offset, mid_point, crate::engine::animation::Smoothing::EaseInOut)
            .translate_last(0.08, [0., 0., 0.], crate::engine::animation::Smoothing::EaseInOut)

    }

    fn build_hurt_anim(direction: Coord2) -> Animation {
        let direction = direction.to_vec2().normalize(12.);
        Animation::new()
            .translate_last(0.02, [direction.x as f64, direction.y as f64, 0.], crate::engine::animation::Smoothing::EaseInOut)
            .translate_last(0.2, [0., 0., 0.], crate::engine::animation::Smoothing::EaseInOut)
    }

    fn build_attack_anim(direction: Coord2) -> Animation {
        let direction = direction.to_vec2().normalize(24.);
        Animation::new()
            .translate_last(0.08, [direction.x as f64, direction.y as f64, 0.], crate::engine::animation::Smoothing::EaseInOut)
            .translate_last(0.08, [0., 0., 0.], crate::engine::animation::Smoothing::EaseInOut)
    }
}

struct RunningAction {
    action: Action,
    actor: usize,
    center: Coord2,
    spell_area: ActionArea,
    // target_actors: Vec<usize>,
    current_step: Option<(RunningActionStep, f64, f64)>,
    steps: VecDeque<RunningActionStep>
}

enum RunningActionStep {
    /// Run the effects of the spell
    Effect(Vec<ActionEffect>),
    /// Spawns an animated sprite
    Sprite(ImageSheetAsset, Coord2),
    /// Plays a sound
    Sound(SoundEffect),
    /// Spawns a projectile
    Projectile(ActionProjectile, Coord2),
    /// Wait
    Wait(f64),
}

#[derive(Debug)]
pub(crate) enum ActionFailReason {
    NotEnoughAP,
    NotEnoughStamina,
    OnCooldown,
    CantReach,
    CantSee,
    NoValidTarget
}