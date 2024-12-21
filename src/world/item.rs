use crate::commons::{damage_model::DamageComponent, history_vec::Id};

use super::world::World;

#[derive(Clone, Debug)]
pub enum Item {
    Sword(Sword),
    Mace(Mace),
    Lance(Lance),
}

impl Item {

    pub fn name(&self, world: &World) -> String {
        match self {
            Item::Sword(sword) => {
                let blade = world.materials.get(&sword.blade_mat).unwrap().name.clone();
                return format!("{blade} sword")
            },
            Item::Mace(mace) => {
                let head = world.materials.get(&mace.head_mat).unwrap().name.clone();
                return format!("{head} mace")
            },
            Item::Lance(lance) => {
                let tip = world.materials.get(&lance.tip_mat).unwrap().name.clone();
                return format!("{tip} lance")
            }
        }
    }

    pub fn description(&self, world: &World) -> String {
        let str;
        match self {
            Item::Sword(sword) => {
                let handle = world.materials.get(&sword.handle_mat).unwrap().name.clone();
                let blade = world.materials.get(&sword.blade_mat).unwrap().name.clone();
                let pommel = world.materials.get(&sword.pommel_mat).unwrap().name.clone();
                let guard = world.materials.get(&sword.guard_mat).unwrap().name.clone();
                str = format!("It's a sword. It's blade is made of {blade}. The handle, made of {handle} is topped by a pomel of {pommel} and a guard of {guard}.");
            },
            Item::Mace(mace) => {
                let handle = world.materials.get(&mace.handle_mat).unwrap().name.clone();
                let head = world.materials.get(&mace.head_mat).unwrap().name.clone();
                let pommel = world.materials.get(&mace.pommel_mat).unwrap().name.clone();
                str = format!("It's a mace. It's head is made of {head}. The handle, made of {handle} is topped by a pomel of {pommel}.");
            },
            Item::Lance(lance) => {
                let handle = world.materials.get(&lance.handle_mat).unwrap().name.clone();
                let tip = world.materials.get(&lance.tip_mat).unwrap().name.clone();
                str = format!("It's a lance. It's tip is made of {tip}, with a handle made of {handle}.");
            }
        }
        return str
    }

    pub fn damage_model(&self) -> DamageComponent {
        match self {
            Item::Sword(sword) => sword.damage,
            Item::Mace(mace) => mace.damage,
            Item::Lance(lance) => lance.damage
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sword {
    pub handle_mat: Id,
    pub blade_mat: Id,
    pub pommel_mat: Id,
    pub guard_mat: Id,
    pub damage: DamageComponent
}

impl Sword {
    pub fn new(handle_mat: Id, blade_mat: Id, pommel_mat: Id, guard_mat: Id, world: &World) -> Sword {
        let blade = world.materials.get(&blade_mat).unwrap();
        let damage = DamageComponent::new(blade.sharpness, 0., 0.);
        Sword { handle_mat, blade_mat, pommel_mat, guard_mat, damage }
    }
}

#[derive(Clone, Debug)]
pub struct Mace {
    pub handle_mat: Id,
    pub head_mat: Id,
    pub pommel_mat: Id,
    pub damage: DamageComponent
}

impl Mace {
    pub fn new(handle_mat: Id, head_mat: Id, pommel_mat: Id, world: &World) -> Mace {
        let head = world.materials.get(&head_mat).unwrap();
        let damage = DamageComponent::new(0., 0., head.sharpness);
        Mace { handle_mat, head_mat, pommel_mat, damage }
    }
}

#[derive(Clone, Debug)]
pub struct Lance {
    pub handle_mat: Id,
    pub tip_mat: Id,
    pub damage: DamageComponent
}

impl Lance {
    pub fn new(handle_mat: Id, tip_mat: Id, world: &World) -> Lance {
        let tip = world.materials.get(&tip_mat).unwrap();
        let damage = DamageComponent::new(0., tip.sharpness, 0.);
        Lance { handle_mat, tip_mat, damage }
    }
}