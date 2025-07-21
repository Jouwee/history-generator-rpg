use crate::{commons::damage_model::DamageComponent, Rng};

use super::{actor_stats::ActorStats, health_component::BodyPart};

pub(crate) fn resolve_damage(damage: &DamageComponent, attacker_stats: &ActorStats<'_>, target: &BodyPart, defender_stats: &ActorStats<'_>) -> DamageOutput {
    let mut rng = Rng::rand();
    let mut damage = damage.clone();

    if rng.rand_chance(defender_stats.dodge_chance() + attacker_stats.enemy_dodge_bonus()) {
        return DamageOutput::Dodged
    }

    let protection = defender_stats.protection(target);
    damage = damage - protection;

    let random = rng.randf_range(0.85, 1.15);
    damage = damage.multiply(random);

    let total_damage = damage.slashing + damage.bludgeoning + damage.piercing + damage.arcane + damage.fire;

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