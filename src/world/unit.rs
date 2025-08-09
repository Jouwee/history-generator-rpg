use std::ops::Add;

use crate::{commons::{id_vec::IdVec, rng::Rng}, engine::geometry::Coord2, resources::material::MaterialId};

use super::{creature::{CreatureId, Profession}, item::ItemId};

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
    pub(crate) name: Option<String>,
    pub(crate) creatures: Vec<CreatureId>,
    pub(crate) cemetery: Vec<CreatureId>,
    pub(crate) resources: UnitResources,
    pub(crate) settlement: Option<SettlementComponent>,
    pub(crate) artifacts: Vec<ItemId>,
    pub(crate) population_peak: (i32, u32),
    pub(crate) unit_type: UnitType
}

impl Unit {

    pub(crate) fn name(&self) -> &str {
        if let Some(name) = &self.name {
            return name.as_str();
        }
        match &self.unit_type {
            UnitType::BanditCamp => "Bandit camp",
            UnitType::VarningrLair => "Varningr lair",
            UnitType::WolfPack => "Wolf den",
            UnitType::Village => "Village",
        }
    }

    pub(crate) fn remove_creature(&mut self, id: &CreatureId) {
        if let Some(idx) = self.creatures.iter().position(|another| another == id) {
            self.creatures.remove(idx);
        }
    }

    pub(crate) fn select_new_profession(&self, rng: &mut Rng) -> Profession {
        match self.unit_type {
            UnitType::BanditCamp => Profession::Bandit,
            UnitType::Village => {
                // Ideally this would look at what the city needs
                let rand_job = rng.randf();
                if rand_job < 0.8 {
                    return Profession::Peasant;
                } else if rand_job < 0.88 {
                    return Profession::Farmer;
                } else if rand_job < 0.90 {
                    return Profession::Sculptor;
                } else if rand_job < 0.95 {
                    return Profession::Blacksmith;
                } else {
                    return Profession::Guard;
                }
            },
            UnitType::VarningrLair => Profession::Beast,
            UnitType::WolfPack => Profession::Beast,
        }
    }

}

pub(crate) struct SettlementComponent {
    pub(crate) leader: Option<CreatureId>,
    pub(crate) material_stock: Vec<(MaterialId, usize)>
}

impl SettlementComponent {

    pub(crate) fn add_material(&mut self, material: &MaterialId, number: usize) {
        let i = self.material_stock.iter().position(|(id, _c)| id == material);
        if let Some(i) = i {
            self.material_stock[i].1 += number;
        } else {
            self.material_stock.push((*material, number));
        }
    }

}

#[cfg(test)]
mod tests_unit {
    use crate::commons::id_vec::Id;
    use super::*;

    #[test]
    fn test_remove_creature() {
        let mut unit = Unit {
            xy: Coord2::xy(0, 0),
            creatures: Vec::new(),
            cemetery: Vec::new(),
            resources: UnitResources {
                food: 0.
            },
            name: None,
            settlement: None,
            artifacts: Vec::new(),
            population_peak: (0, 0),
            unit_type: UnitType::Village
        };
        unit.creatures.push(CreatureId::mock(0));
        unit.creatures.push(CreatureId::mock(1));
        unit.remove_creature(&CreatureId::mock(0));
        assert_eq!(unit.creatures.len(), 1);
        assert_eq!(unit.creatures[0].as_usize(), 1);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UnitType {
    Village,
    BanditCamp,
    WolfPack,
    VarningrLair,
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