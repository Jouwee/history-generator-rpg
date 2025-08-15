use std::{fmt::Display, ops::{Add, Sub}};

use serde::{Deserialize, Serialize};

use crate::commons::rng::Rng;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct DamageRoll {
    pub(crate) slashing: f32,
    pub(crate) piercing: f32,
    pub(crate) bludgeoning: f32,
    pub(crate) fire: f32,
    pub(crate) arcane: f32,
}

impl DamageRoll {

    pub(crate) fn empty() -> Self {
        DamageRoll {
            slashing: 0.,
            piercing: 0.,
            bludgeoning: 0.,
            fire: 0.,
            arcane: 0.
        }
    }

    pub(crate) fn slashing(damage: f32) -> Self {
        let mut roll = Self::empty();
        roll.slashing = damage;
        return roll;
    }

    pub(crate) fn piercing(damage: f32) -> Self {
        let mut roll = Self::empty();
        roll.piercing = damage;
        return roll;
    }

    pub(crate) fn bludgeoning(damage: f32) -> Self {
        let mut roll = Self::empty();
        roll.bludgeoning = damage;
        return roll;
    }

    pub(crate) fn fire(damage: f32) -> Self {
        let mut roll = Self::empty();
        roll.fire = damage;
        return roll;
    }

    pub(crate) fn arcane(damage: f32) -> Self {
        let mut roll = Self::empty();
        roll.arcane = damage;
        return roll;
    }

    pub(crate) fn roll(&self) -> DamageModel {
        return DamageModel {
            slashing: roll_f(self.slashing),
            piercing: roll_f(self.piercing),
            bludgeoning: roll_f(self.bludgeoning),
            fire: roll_f(self.fire),
            arcane: roll_f(self.arcane),
        }
    }

    pub(crate) fn average(&self) -> f32 {
        return 
            self.slashing +
            self.piercing +
            self.bludgeoning +
            self.fire +
            self.arcane
    }

    pub(crate) fn multiply(&self, factor: f32) -> Self {
        return Self {
            slashing: self.slashing * factor,
            piercing: self.piercing * factor,
            bludgeoning: self.bludgeoning * factor,
            fire: self.fire * factor,
            arcane: self.arcane * factor,
        }
    }

    pub(crate) fn to_string(&self) -> String {
        let mut parts = Vec::new();
        if self.slashing > 0. {
            parts.push(format_f(self.slashing) + " slashing");
        }
        if self.piercing > 0. {
            parts.push(format_f(self.piercing) + " piercing");
        }
        if self.bludgeoning > 0. {
            parts.push(format_f(self.bludgeoning) + " bludgeoning");
        }
        if self.fire > 0. {
            parts.push(format_f(self.fire) + " fire");
        }
        if self.arcane > 0. {
            parts.push(format_f(self.arcane) + " arcane");
        }
        return parts.join(" + ");
    }
 
}

fn roll_f(f: f32) -> i32 {
    let mut rng = Rng::rand();
    return (f * rng.randf_range(0.75, 1.25)).round() as i32;
}

fn format_f(f: f32) -> String {
    let min = f * 0.75;
    let max = f * 1.25;
    return format!("{:.0}-{:.0}", min, max);
}

impl Add for DamageRoll {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            slashing: self.slashing + rhs.slashing,
            piercing: self.piercing + rhs.piercing,
            bludgeoning: self.bludgeoning + rhs.bludgeoning,
            fire: self.fire + rhs.fire,
            arcane: self.arcane + rhs.arcane,
        }
    }

}

#[cfg(test)]
mod tests_damage_roll {
    use super::*;

    #[test]
    fn test_add() {
        let roll = DamageRoll::slashing(10.) + DamageRoll::fire(20.);
        assert!(roll.average() == 30.);
    }

    #[test]
    fn to_string() {
        let roll = DamageRoll::slashing(20.) + DamageRoll::fire(10.);
        assert_eq!(roll.to_string(), "15-25 slashing + 7.5-12.5 fire");
    }

}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct DamageModel {
    pub(crate) slashing: i32,
    pub(crate) piercing: i32,
    pub(crate) bludgeoning: i32,
    pub(crate) fire: i32,
    pub(crate) arcane: i32,
}

impl DamageModel {

    pub(crate) fn new() -> Self {
        Self { slashing: 0, piercing: 0, bludgeoning: 0, fire: 0, arcane: 0 }
    }

    pub(crate) fn new_spb(slashing: i32, piercing: i32, bludgeoning: i32) -> Self {
        DamageModel { slashing, piercing, bludgeoning, fire: 0, arcane: 0 }
    }

    pub(crate) fn multiply(&self, factor: f32) -> Self {
        return Self {
            slashing: (self.slashing as f32 * factor).round() as i32,
            piercing: (self.piercing as f32 * factor).round() as i32,
            bludgeoning: (self.bludgeoning as f32 * factor).round() as i32,
            fire: (self.fire as f32 * factor).round() as i32,
            arcane: (self.arcane as f32 * factor).round() as i32,
        }
    }

}

impl Add for DamageModel {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            slashing: self.slashing + rhs.slashing,
            piercing: self.piercing + rhs.piercing,
            bludgeoning: self.bludgeoning + rhs.bludgeoning,
            fire: self.fire + rhs.fire,
            arcane: self.arcane + rhs.arcane,
        }
    }

}

impl Sub for DamageModel {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            slashing: (self.slashing - rhs.slashing).max(0),
            piercing: (self.piercing - rhs.piercing).max(0),
            bludgeoning: (self.bludgeoning - rhs.bludgeoning).max(0),
            fire: (self.fire - rhs.fire).max(0),
            arcane: (self.arcane - rhs.arcane).max(0),
        }
    }

}

impl Display for DamageModel {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_str(&format!("{:1}/{:1}/{:1}", self.slashing, self.piercing, self.bludgeoning));
    }

}