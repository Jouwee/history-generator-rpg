use crate::{commons::damage_model::{DamageComponent, DamageOutput}, engine::{animation::Animation, audio::SoundEffect, geometry::Coord2, Palette}, GameContext};

use super::{actor::Actor, chunk::ChunkMap, effect_layer::EffectLayer};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct ActionId(usize);
impl crate::commons::id_vec::Id for ActionId {
    fn new(id: usize) -> Self {
        ActionId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone)]
pub struct Action {
    pub name: String,
    pub icon: String,
    pub description: String,
    pub sound_effect: Option<SoundEffect>,
    pub ap_cost: u16,
    pub action_type: ActionType
}

#[derive(Clone)]
pub enum ActionType {
    Move { offset: Coord2 },
    Targeted {
        damage: Option<DamageType>,
        inflicts: Option<Infliction>
    },
    Talk,
    PickUp,
    Sleep
}

#[derive(Clone)]
pub enum DamageType {
    FromWeapon(DamageComponent),
    Fixed(DamageComponent)
}

#[derive(Clone)]
pub struct Infliction {
    pub chance: AfflictionChance,
    pub affliction: Affliction,
}

#[derive(Clone)]
pub enum AfflictionChance {
    OnHit
}

#[derive(Clone)]
pub enum Affliction {
    Bleeding { duration: usize },
    Poisoned { duration: usize },
    Stunned { duration: usize }
}

impl Affliction {
    pub fn name_color(&self) -> (&str, Palette) {
        match self {
            Affliction::Bleeding { duration: _ } => ("Bleeding", Palette::Red),
            Affliction::Poisoned { duration: _ } => ("Poisoned", Palette::Green),
            Affliction::Stunned { duration: _ } => ("Stunned", Palette::Gray),
        }
    }
}

pub struct ActionRunner { }

impl ActionRunner {
    pub fn move_try_use(action: &Action, actor: &mut Actor, chunk_map: &ChunkMap, ctx: &GameContext, player_pos: &Coord2) -> bool {
        match &action.action_type {
            ActionType::Move { offset } => {
                if actor.ap.can_use(action.ap_cost) {
                    let xy = actor.xy.clone();
                    let pos = xy + *offset;
                    if !chunk_map.blocks_movement(pos) {
                        actor.ap.consume(action.ap_cost);
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

    pub fn targeted_try_use(action: &Action, actor: &mut Actor, target: &mut Actor, effect_layer: &mut EffectLayer, ctx: &GameContext) -> bool {
        match &action.action_type {
            ActionType::Targeted { damage, inflicts } => {
                if actor.ap.can_use(action.ap_cost) {
                    if actor.xy.dist_squared(&target.xy) < 3. {
                        actor.ap.consume(action.ap_cost);
                        let mut hit = true;
                        if let Some(damage) = damage {
                            // Compute damage
                            let damage = match &damage {
                                DamageType::Fixed(dmg) => dmg,
                                DamageType::FromWeapon(dmg) => {
                                    let item = actor.inventory.equipped().expect("Used equipped action with no equipped item");
                                    &dmg.multiply(item.damage_mult())
                                }
                            };
                            let str_mult = actor.attributes.strength_attack_damage_mult();
                            let damage_model = damage.multiply(str_mult);
                            let damage = damage_model.resolve(&target.defence);

                            match damage {
                                DamageOutput::Dodged => {
                                    hit = false;
                                    effect_layer.add_text_indicator(target.xy, "Dodged", Palette::Gray);
                                },
                                DamageOutput::Hit(damage) => {
                                    target.hp.damage(damage);
                                    effect_layer.add_damage_number(target.xy, damage);
                                }
                            }

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