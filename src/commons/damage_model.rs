use std::{fmt::Display, ops::{Add, Sub}};

// TODO: Rename to Damage Model. Will represent damage, defence, etc
#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct DamageComponent {
    pub(crate) slashing: f32,
    pub(crate) piercing: f32,
    pub(crate) bludgeoning: f32,
    pub(crate) fire: f32,
    pub(crate) arcane: f32,
}

impl DamageComponent {

    pub(crate) fn new(slashing: f32, piercing: f32, bludgeoning: f32) -> DamageComponent {
        DamageComponent { 
            slashing: slashing.max(0.),
            piercing: piercing.max(0.),
            bludgeoning: bludgeoning.max(0.),
            fire: 0.,
            arcane: 0.
        }
    }

    pub(crate) fn arcane(arcane: f32) -> DamageComponent {
        DamageComponent { 
            slashing: 0.,
            piercing: 0.,
            bludgeoning: 0.,
            fire: 0.,
            arcane: arcane.max(0.),
        }
    }

    pub(crate) fn multiply(&self, mult: f32) -> DamageComponent {
        DamageComponent {
            slashing: self.slashing * mult,
            piercing: self.piercing * mult,
            bludgeoning: self.bludgeoning * mult,
            fire: self.fire * mult,
            arcane: self.arcane * mult,
        }
    }
}

impl Add for DamageComponent {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        DamageComponent{
            slashing: self.slashing + rhs.slashing,
            piercing: self.piercing + rhs.piercing,
            bludgeoning: self.bludgeoning + rhs.bludgeoning,
            fire: self.fire + rhs.fire,
            arcane: self.arcane + rhs.arcane,
        }
    }

}

impl Sub for DamageComponent {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        DamageComponent{
            slashing: self.slashing - rhs.slashing,
            piercing: self.piercing - rhs.piercing,
            bludgeoning: self.bludgeoning - rhs.bludgeoning,
            fire: self.fire - rhs.fire,
            arcane: self.arcane - rhs.arcane,
        }
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