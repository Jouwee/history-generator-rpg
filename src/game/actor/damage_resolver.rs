use crate::{commons::damage_model::DamageRoll, Rng};

use super::{actor_stats::ActorStats, health_component::BodyPart};

pub(crate) fn resolve_damage(damage: &DamageRoll, attacker_stats: &ActorStats<'_>, target: &BodyPart, defender_stats: &ActorStats<'_>) -> DamageOutput {
    let mut rng = Rng::rand();

    if rng.rand_chance(defender_stats.dodge_chance() + attacker_stats.enemy_dodge_bonus()) {
        return DamageOutput::Dodged
    }

    let mut damage = damage.roll();

    let protection = defender_stats.protection(target);
    damage = damage - protection;

    let total_damage = damage.slashing + damage.bludgeoning + damage.piercing + damage.arcane + damage.fire;

    let total_damage = total_damage as f32;

    if rng.rand_chance(attacker_stats.critical_hit_chance()) {
        return DamageOutput::CriticalHit(total_damage * attacker_stats.critical_hit_multiplier());    
    }
    return DamageOutput::Hit(total_damage);
}

#[derive(Debug)]
pub(crate) enum DamageOutput {
    Dodged,
    Hit(f32),
    CriticalHit(f32),
}