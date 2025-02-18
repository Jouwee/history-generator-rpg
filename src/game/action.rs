use crate::{commons::damage_model::DamageComponent, engine::{audio::SoundEffect, geometry::Coord2}};


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct ActionId(usize);
impl crate::commons::id_vec::Id for ActionId {
    fn new(id: usize) -> Self {
        ActionId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone)]
pub struct Action {
    pub name: String,
    pub icon: String,
    pub sound_effect: Option<SoundEffect>,
    pub ap_cost: u16,
    pub action_type: ActionType
}

#[derive(Clone)]
pub enum ActionType {
    Move { offset: Coord2 },
    Targeted { damage: Option<DamageType> },
    Talk,
    PickUp,
    Sleep
}

#[derive(Clone)]
pub enum DamageType {
    FromWeapon,
    Fixed(DamageComponent)
}

pub struct ActionTargetOutput {
    pub damage: Option<DamageOutput>
}


pub struct DamageOutput {
    pub damage: f32
}
