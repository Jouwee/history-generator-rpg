#[derive(Debug, Clone)]
pub struct Attributes {
    pub unallocated: u8,
    pub strength: u8,
    pub agility: u8,
    pub constitution: u8
}

impl Attributes {

    pub fn simplified_offensive_power(&self) -> f32 {
        return 2. * self.strength_attack_damage_mult()
    }

    pub fn simplified_health(&self) -> f32 {
        return 10. + self.bonus_hp() as f32;
    }

    pub fn strength_attack_damage_mult(&self) -> f32 {
        self.strength as f32 / 10.
    }

    pub fn bonus_ap(&self) -> i32 {
        self.agility as i32 - 10
    }

    pub fn bonus_hp(&self) -> i32 {
        self.constitution as i32 - 10
    }

    pub fn dodge_chance(&self) -> f32 {
        return self.agility as f32 * 0.01
    }

}

#[derive(Clone)]
pub enum AttributeEnum {
    Strength,
    Agility,
    Constitution
}