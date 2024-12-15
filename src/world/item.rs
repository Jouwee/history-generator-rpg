use crate::commons::history_vec::Id;

#[derive(Debug)]
pub enum Item {
    Sword(Sword),
    Mace(Mace),
    Lance(Lance),
}

#[derive(Debug)]
pub struct Sword {
    pub handle_mat: Id,
    pub blade_mat: Id,
    pub pomel_mat: Id,
    pub guard_mat: Id
}

#[derive(Debug)]
pub struct Mace {
    pub handle_mat: Id,
    pub head_mat: Id,
    pub pomel_mat: Id,
}

#[derive(Debug)]
pub struct Lance {
    pub handle_mat: Id,
    pub tip_mat: Id,
}