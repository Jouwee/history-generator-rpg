use std::ops::Add;

use crate::{commons::id_vec::IdVec, engine::geometry::Coord2};

use super::{creature::CreatureId, world::ArtifactId};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct UnitId(usize);
impl UnitId {
    pub fn ancients() -> UnitId {
        return UnitId(0);
    }
}
impl crate::commons::id_vec::Id for UnitId {
    fn new(id: usize) -> Self {
        UnitId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub type Units = IdVec<Unit>;

pub struct Unit {
    pub xy: Coord2,
    pub creatures: Vec<CreatureId>,
    pub cemetery: Vec<CreatureId>,
    pub unit_type: UnitType,
    pub resources: UnitResources,
    pub leader: Option<CreatureId>,
    pub artifacts: Vec<ArtifactId>
}

pub enum UnitType {
    City,
}

#[derive(Clone, Copy)]
pub struct UnitResources {
    // 1 unit = enough food for 1 adult for 1 year
    pub food: f32,
}

impl Add for UnitResources {
    type Output = UnitResources;

    fn add(self, other: UnitResources) -> UnitResources {
        return UnitResources {
            food: self.food + other.food
        }
    }
}
