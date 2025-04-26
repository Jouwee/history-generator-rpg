use std::ops::Add;

use crate::{commons::id_vec::IdVec, engine::geometry::Coord2};

use super::{creature::CreatureId, world::ArtifactId};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct UnitId(usize);
impl crate::commons::id_vec::Id for UnitId {
    fn new(id: usize) -> Self {
        UnitId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Units = IdVec<Unit>;

pub(crate) struct Unit {
    pub(crate) xy: Coord2,
    pub(crate) creatures: Vec<CreatureId>,
    pub(crate) cemetery: Vec<CreatureId>,
    pub(crate) unit_type: UnitType,
    pub(crate) resources: UnitResources,
    pub(crate) leader: Option<CreatureId>,
    pub(crate) artifacts: Vec<ArtifactId>
}

pub(crate) enum UnitType {
    City,
}

#[derive(Clone, Copy)]
pub(crate) struct UnitResources {
    // 1 unit = enough food for 1 adult for 1 year
    pub(crate) food: f32,
}

impl Add for UnitResources {
    type Output = UnitResources;

    fn add(self, other: UnitResources) -> UnitResources {
        return UnitResources {
            food: self.food + other.food
        }
    }
}
