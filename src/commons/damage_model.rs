use std::ops::Add;

use crate::world::attributes::Attributes;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DamageComponent {
    slashing: f32,
    piercing: f32,
    bludgeoning: f32,
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

    pub fn resolve(&self, defence: &DefenceComponent) -> f32 {
        return (self.slashing - defence.slashing).max(0.0) + (self.piercing - defence.piercing).max(0.0) + (self.bludgeoning - defence.bludgeoning).max(0.0)
    }

}

impl Add for DamageComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DamageComponent::new(self.slashing + rhs.slashing, self.piercing + rhs.piercing, self.bludgeoning + rhs.bludgeoning)
    }

}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DefenceComponent {
    pub slashing: f32,
    pub piercing: f32,
    pub bludgeoning: f32,
}


impl DefenceComponent {

    pub fn new(slashing: f32, piercing: f32, bludgeoning: f32) -> DefenceComponent {
        DefenceComponent { slashing, piercing, bludgeoning }
    }

}

impl Add for DefenceComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DefenceComponent::new(self.slashing + rhs.slashing, self.piercing + rhs.piercing, self.bludgeoning + rhs.bludgeoning)
    }

}