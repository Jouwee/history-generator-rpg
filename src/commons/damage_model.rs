use std::ops::Add;

use super::rng::Rng;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct DamageComponent {
    pub(crate) slashing: f32,
    pub(crate) piercing: f32,
    pub(crate) bludgeoning: f32,
}

impl DamageComponent {

    pub(crate) fn new(slashing: f32, piercing: f32, bludgeoning: f32) -> DamageComponent {
        DamageComponent { slashing, piercing, bludgeoning }
    }

    pub(crate) fn resolve(&self, defence: &DefenceComponent) -> DamageOutput {
        let mut total_damage = 0.;
        let mut rng = Rng::rand();

        if rng.rand_chance(defence.dodge_chance) {
            return DamageOutput::Dodged
        }

        if self.slashing > 0. {
            let slashing = (self.slashing * rng.randf_range(0.85, 1.15)) - defence.slashing;
            total_damage += slashing.max(0.);
        }

        if self.piercing > 0. {
            let piercing = (self.piercing * rng.randf_range(0.85, 1.15)) - defence.piercing;
            total_damage += piercing.max(0.);
        }

        if self.bludgeoning > 0. {
            let bludgeoning = (self.bludgeoning * rng.randf_range(0.85, 1.15)) - defence.bludgeoning;
            total_damage += bludgeoning.max(0.);
        }

        if rng.rand_chance(0.05) {
            return DamageOutput::CriticalHit(total_damage * 2.);    
        }
        return DamageOutput::Hit(total_damage);
    }

    pub(crate) fn multiply(&self, mult: f32) -> DamageComponent {
        DamageComponent {
            slashing: self.slashing * mult,
            piercing: self.piercing * mult,
            bludgeoning: self.bludgeoning * mult
        }
    }

}

pub(crate) enum DamageOutput {
    Dodged,
    Hit(f32),
    CriticalHit(f32),
}

impl Add for DamageComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DamageComponent::new(self.slashing + rhs.slashing, self.piercing + rhs.piercing, self.bludgeoning + rhs.bludgeoning)
    }

}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct DefenceComponent {
    pub(crate) dodge_chance: f32,
    pub(crate) slashing: f32,
    pub(crate) piercing: f32,
    pub(crate) bludgeoning: f32,
}


impl DefenceComponent {

    pub(crate) fn new(slashing: f32, piercing: f32, bludgeoning: f32, dodge_chance: f32) -> DefenceComponent {
        DefenceComponent { slashing, piercing, bludgeoning, dodge_chance }
    }

}

impl Add for DefenceComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DefenceComponent::new(self.slashing + rhs.slashing, self.piercing + rhs.piercing, self.bludgeoning + rhs.bludgeoning, self.dodge_chance + rhs.dodge_chance)
    }

}