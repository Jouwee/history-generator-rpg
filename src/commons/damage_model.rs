use std::{fmt::Display, ops::{Add, Sub}};

// TODO: Rename to Damage Model. Will represent damage, defence, etc
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct DamageComponent {
    pub(crate) slashing: f32,
    pub(crate) piercing: f32,
    pub(crate) bludgeoning: f32,
}

impl DamageComponent {

    pub(crate) fn new(slashing: f32, piercing: f32, bludgeoning: f32) -> DamageComponent {
        DamageComponent { 
            slashing: slashing.max(0.),
            piercing: piercing.max(0.),
            bludgeoning: bludgeoning.max(0.)
        }
    }

    pub(crate) fn multiply(&self, mult: f32) -> DamageComponent {
        DamageComponent {
            slashing: self.slashing * mult,
            piercing: self.piercing * mult,
            bludgeoning: self.bludgeoning * mult
        }
    }
}

impl Add for DamageComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DamageComponent::new(self.slashing + rhs.slashing, self.piercing + rhs.piercing, self.bludgeoning + rhs.bludgeoning)
    }

}

impl Sub for DamageComponent {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        DamageComponent::new(self.slashing - rhs.slashing, self.piercing - rhs.piercing, self.bludgeoning - rhs.bludgeoning)
    }

}

impl Display for DamageComponent {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str(&format!("{:1}/{:1}/{:1}", self.slashing, self.piercing, self.bludgeoning));
    }

}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct DefenceComponent {
    pub(crate) slashing: f32,
    pub(crate) piercing: f32,
    pub(crate) bludgeoning: f32,
}


impl DefenceComponent {

    pub(crate) fn new(slashing: f32, piercing: f32, bludgeoning: f32) -> DefenceComponent {
        DefenceComponent { slashing, piercing, bludgeoning }
    }

}

impl Add for DefenceComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DefenceComponent::new(self.slashing + rhs.slashing, self.piercing + rhs.piercing, self.bludgeoning + rhs.bludgeoning)
    }

}