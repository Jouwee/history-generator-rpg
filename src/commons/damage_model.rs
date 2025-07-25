use std::{fmt::Display, ops::{Add, Sub}};

use regex::Regex;

use crate::commons::rng::Rng;

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) struct Roll {
    pub(crate) count: u8,
    pub(crate) dice: u8,
    pub(crate) modifier: i16
}

impl Roll {

    pub(crate) fn parse(string: &str) -> Self {
        let re = Regex::new(r"(\d*)d(\d+)((?:[-+]\d+)?)").unwrap();
        let (_, [count, dice, modifier]) = re.captures_iter(string).next().unwrap().extract();
        let count = match count.is_empty() {
            true => 1,
            false => count.parse::<u8>().unwrap()
        };
        let dice = dice.parse::<u8>().unwrap();
        let modifier = match modifier.is_empty() {
            true => 0,
            false => modifier.parse::<i16>().unwrap()
        };
        Self { count, dice, modifier }
    }

    pub(crate) fn average(&self) -> f32 {
        return ((self.dice as f32 / 2.) + 0.5) * self.count as f32 + self.modifier as f32
    }

    pub(crate) fn roll(&self) -> i32 {
        let mut rng = Rng::rand();
        let mut roll = self.modifier as i32;
        for _ in 0..self.count {
            roll = roll + rng.randi_range(0, self.dice as i32) + 1;
        }
        return roll;
    }

    pub(crate) fn to_string(&self) -> String {
        let mut string = format!("{}d{}", self.count, self.dice);
        if self.modifier > 0 {
            string = string + &format!("+{}", self.modifier)
        }
        if self.modifier < 0 {
            string = string + &format!("{}", self.modifier)
        }
        return string
    }

}

#[cfg(test)]
mod tests_roll {
    use super::*;

    #[test]
    fn test_roll() {
        let roll = Roll::parse("3d6+1");
        assert_eq!(roll.average(), 11.5);
        let v = roll.roll();
        assert!(v >= 3 && v <= 18);

        let roll = Roll::parse("d8");
        assert_eq!(roll.average(), 4.5);
        let v = roll.roll();
        assert!(v >= 1 && v <= 8);
    }

}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DamageRoll {
    pub(crate) slashing: Vec<Roll>,
    pub(crate) piercing: Vec<Roll>,
    pub(crate) bludgeoning: Vec<Roll>,
    pub(crate) fire: Vec<Roll>,
    pub(crate) arcane: Vec<Roll>,
}

impl DamageRoll {

    pub(crate) fn empty() -> Self {
        DamageRoll {
            slashing: Vec::new(),
            piercing: Vec::new(),
            bludgeoning: Vec::new(),
            fire: Vec::new(),
            arcane: Vec::new()
        }
    }

    pub(crate) fn slashing(string: &str) -> Self {
        let mut roll = Self::empty();
        roll.slashing.push(Roll::parse(string));
        return roll;
    }

    pub(crate) fn piercing(string: &str) -> Self {
        let mut roll = Self::empty();
        roll.piercing.push(Roll::parse(string));
        return roll;
    }

    pub(crate) fn bludgeoning(string: &str) -> Self {
        let mut roll = Self::empty();
        roll.bludgeoning.push(Roll::parse(string));
        return roll;
    }

    pub(crate) fn fire(string: &str) -> Self {
        let mut roll = Self::empty();
        roll.fire.push(Roll::parse(string));
        return roll;
    }

    pub(crate) fn arcane(string: &str) -> Self {
        let mut roll = Self::empty();
        roll.arcane.push(Roll::parse(string));
        return roll;
    }

    pub(crate) fn add_modifier(&mut self, modifier: i16) {
        if let Some(slashing) = self.slashing.first_mut() {
            slashing.modifier += modifier;
        }
        if let Some(piercing) = self.piercing.first_mut() {
            piercing.modifier += modifier;
        }
        if let Some(bludgeoning) = self.bludgeoning.first_mut() {
            bludgeoning.modifier += modifier;
        }
        if let Some(fire) = self.fire.first_mut() {
            fire.modifier += modifier;
        }
        if let Some(arcane) = self.arcane.first_mut() {
            arcane.modifier += modifier;
        }
    }

    pub(crate) fn roll(&self) -> DamageModel {
        return DamageModel {
            slashing: self.slashing.iter().fold(0, |acc, r| acc + r.roll()),
            piercing: self.piercing.iter().fold(0, |acc, r| acc + r.roll()),
            bludgeoning: self.bludgeoning.iter().fold(0, |acc, r| acc + r.roll()),
            fire: self.fire.iter().fold(0, |acc, r| acc + r.roll()),
            arcane: self.arcane.iter().fold(0, |acc, r| acc + r.roll()),
        }
    }

    pub(crate) fn average(&self) -> f32 {
        return 
            self.slashing.iter().fold(0., |acc, r| acc + r.average()) +
            self.piercing.iter().fold(0., |acc, r| acc + r.average()) +
            self.bludgeoning.iter().fold(0., |acc, r| acc + r.average()) +
            self.fire.iter().fold(0., |acc, r| acc + r.average()) +
            self.arcane.iter().fold(0., |acc, r| acc + r.average())
    }

    pub(crate) fn to_string(&self) -> String {
        let mut parts = Vec::new();
        for slashing in self.slashing.iter() {
            parts.push(slashing.to_string() + " slashing");
        }
        for piercing in self.piercing.iter() {
            parts.push(piercing.to_string() + " piercing");
        }
        for bludgeoning in self.bludgeoning.iter() {
            parts.push(bludgeoning.to_string() + " bludgeoning");
        }
        for fire in self.fire.iter() {
            parts.push(fire.to_string() + " fire");
        }
        for arcane in self.arcane.iter() {
            parts.push(arcane.to_string() + " arcane");
        }
        return parts.join(" + ");
    }
 
}

impl Add for DamageRoll {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            slashing: self.slashing.iter().cloned().chain(rhs.slashing.iter().cloned()).collect(),
            piercing: self.piercing.iter().cloned().chain(rhs.piercing.iter().cloned()).collect(),
            bludgeoning: self.bludgeoning.iter().cloned().chain(rhs.bludgeoning.iter().cloned()).collect(),
            fire: self.fire.iter().cloned().chain(rhs.fire.iter().cloned()).collect(),
            arcane: self.arcane.iter().cloned().chain(rhs.arcane.iter().cloned()).collect(),
        }
    }

}

#[cfg(test)]
mod tests_damage_roll {
    use super::*;

    #[test]
    fn test_add() {
        let roll = DamageRoll::slashing("1d6") + DamageRoll::fire("1d6");
        assert!(roll.slashing.len() == 1);
        assert!(roll.fire.len() == 1);
    }

    #[test]
    fn test_add_modifier() {
        let mut roll = DamageRoll::slashing("1d6+1");
        roll.add_modifier(2);
        assert!(roll.slashing.len() == 1);
        assert_eq!(roll.slashing.get(0).unwrap().modifier, 3);
    }

    #[test]
    fn to_string() {
        let roll = DamageRoll::slashing("2d6+1") + DamageRoll::fire("1d4");
        assert_eq!(roll.to_string(), "2d6+1 slashing + 1d4 fire");
    }

}

#[derive(Debug, Copy, Clone, PartialEq)]
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