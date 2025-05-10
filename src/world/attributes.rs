#[derive(Debug, Clone)]
pub(crate) struct Attributes {
    pub(crate) unallocated: u8,
    pub(crate) strength: u8,
    pub(crate) agility: u8,
    pub(crate) constitution: u8
}

impl Attributes {

    pub(crate) fn strength_attack_damage_mult(&self) -> f32 {
        self.strength as f32 / 10.
    }

    pub(crate) fn bonus_ap(&self) -> i32 {
        self.agility as i32 - 10
    }

}