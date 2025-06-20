use std::ops::Add;

use crate::{commons::{id_vec::IdVec, rng::Rng}, engine::geometry::Coord2, resources::material::MaterialId};

use super::{creature::{Creature, CreatureId, Profession}, date::WorldDate, item::ItemId};

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
    pub(crate) resources: UnitResources,
    pub(crate) settlement: Option<SettlementComponent>,
    pub(crate) artifacts: Vec<ItemId>,
    pub(crate) population_peak: (i32, u32),
    pub(crate) unit_type: UnitType
}

impl Unit {

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
            }
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

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum UnitType {
    Village,
    BanditCamp,
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


pub(crate) struct Demographics {
    total: u16,
    children_male: u16,
    children_female: u16,
    adult_male: u16,
    adult_female: u16,
    adult_singles: u16,
    adult_married: u16,
    employed: u16,
    army: u16,
    peasants: u16,
    farmers: u16,
    artisans: u16,
    politicians: u16,
    outlaws: u16,
}

impl Demographics {

    pub(crate) fn new() -> Demographics {
        return Demographics {
            total: 0,
            children_male: 0,
            children_female: 0,
            adult_male: 0,
            adult_female: 0,
            adult_singles: 0,
            adult_married: 0, 
            employed: 0,
            army: 0,
            peasants: 0,
            farmers: 0,
            artisans: 0,
            politicians: 0,
            outlaws: 0,
        }
    }

    pub(crate) fn count(&mut self, reference: &WorldDate, creature: &Creature) {
        let age = (*reference - creature.birth).year();
        self.total += 1;
        if age < 18 {
            if creature.gender.is_male() {
                self.children_male += 1;
            } else {
                self.children_female += 1;
            }
        } else {
            if creature.gender.is_male() {
                self.adult_male += 1;
            } else {
                self.adult_female += 1;
            }
            if creature.spouse.is_none() {
                self.adult_singles += 1;
            } else {
                self.adult_married += 1;
            }
        }
        self.employed += 1;
        match creature.profession {
            Profession::None => self.employed -= 1,
            Profession::Peasant => self.peasants += 1,
            Profession::Farmer => self.farmers += 1,
            Profession::Blacksmith | Profession::Sculptor => self.artisans += 1,
            Profession::Guard => self.army += 1,
            Profession::Ruler => self.politicians += 1,
            Profession::Bandit => self.outlaws += 1,
        }
    }

    pub(crate) fn print_console(&self) {
        println!("total: {}", self.total);
        println!("children_male: {} ({:.2?}%)", self.children_male, Self::pct(self.total, self.children_male));
        println!("children_female: {} ({:.2?}%)", self.children_female, Self::pct(self.total, self.children_female));
        println!("adult_male: {} ({:.2?}%)", self.adult_male, Self::pct(self.total, self.adult_male));
        println!("adult_female: {} ({:.2?}%)", self.adult_female, Self::pct(self.total, self.adult_female));
        println!("adult_singles: {} ({:.2?}%)", self.adult_singles, Self::pct(self.total, self.adult_singles));
        println!("adult_married: {} ({:.2?}%)", self.adult_married, Self::pct(self.total, self.adult_married));
        println!("employed: {} ({:.2?}%)", self.employed, Self::pct(self.total, self.employed));
        println!("army: {} ({:.2?}%)", self.army, Self::pct(self.employed, self.army));
        println!("peasants: {} ({:.2?}%)", self.peasants, Self::pct(self.employed, self.peasants));
        println!("farmers: {} ({:.2?}%)", self.farmers, Self::pct(self.employed, self.farmers));
        println!("artisans: {} ({:.2?}%)", self.artisans, Self::pct(self.employed, self.artisans));
        println!("politicians: {} ({:.2?}%)", self.politicians, Self::pct(self.employed, self.politicians));
        println!("outlaws: {} ({:.2?}%)", self.outlaws, Self::pct(self.employed, self.outlaws));   
    }

    fn pct(total: u16, count: u16) -> f32 {
        return (count as f32 / total as f32) * 100.
    }

}