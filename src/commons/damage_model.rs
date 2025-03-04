use std::ops::Add;

use crate::world::attributes::Attributes;

use super::rng::Rng;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DamageComponent {
    pub slashing: f32,
    pub piercing: f32,
    pub bludgeoning: f32,
}

impl DamageComponent {

    pub fn new(slashing: f32, piercing: f32, bludgeoning: f32) -> DamageComponent {
        DamageComponent { slashing, piercing, bludgeoning }
    }

    pub fn from_attributes(attributes: &Attributes) -> DamageComponent {
        DamageComponent {
            slashing: 0.,
            piercing: 0.,
            bludgeoning: attributes.strength as f32 / 2.
        }
    }

    pub fn resolve(&self, defence: &DefenceComponent) -> DamageOutput {
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

        return DamageOutput::Hit(total_damage);
    }

    pub fn multiply(&self, mult: f32) -> DamageComponent {
        DamageComponent {
            slashing: self.slashing * mult,
            piercing: self.piercing * mult,
            bludgeoning: self.bludgeoning * mult
        }
    }

}

pub enum DamageOutput {
    Dodged,
    Hit(f32)
}

impl Add for DamageComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DamageComponent::new(self.slashing + rhs.slashing, self.piercing + rhs.piercing, self.bludgeoning + rhs.bludgeoning)
    }

}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DefenceComponent {
    pub dodge_chance: f32,
    pub slashing: f32,
    pub piercing: f32,
    pub bludgeoning: f32,
}


impl DefenceComponent {

    pub fn new(slashing: f32, piercing: f32, bludgeoning: f32, dodge_chance: f32) -> DefenceComponent {
        DefenceComponent { slashing, piercing, bludgeoning, dodge_chance }
    }

}

impl Add for DefenceComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DefenceComponent::new(self.slashing + rhs.slashing, self.piercing + rhs.piercing, self.bludgeoning + rhs.bludgeoning, self.dodge_chance + rhs.dodge_chance)
    }

}