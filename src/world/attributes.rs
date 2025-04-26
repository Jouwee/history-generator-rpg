#[derive(Debug, Clone)]
pub(crate) struct Attributes {
    pub(crate) unallocated: u8,
    pub(crate) strength: u8,
    pub(crate) agility: u8,
    pub(crate) constitution: u8
}

impl Attributes {

    pub(crate) fn simplified_offensive_power(&self) -> f32 {
        return 2. * self.strength_attack_damage_mult()
    }

    pub(crate) fn simplified_health(&self) -> f32 {
        return 10. + self.bonus_hp() as f32;
    }

    pub(crate) fn strength_attack_damage_mult(&self) -> f32 {
        self.strength as f32 / 10.
    }

    pub(crate) fn bonus_ap(&self) -> i32 {
        self.agility as i32 - 10
    }

    pub(crate) fn bonus_hp(&self) -> i32 {
        self.constitution as i32 - 10
    }

    pub(crate) fn dodge_chance(&self) -> f32 {
        return self.agility as f32 * 0.01
    }

}

#[derive(Clone)]
pub(crate) enum AttributeEnum {
    Strength,
    Agility,
    Constitution
}