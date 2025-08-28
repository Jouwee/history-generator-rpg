use std::ops::Add;

use math::Vec2i;
use serde::{Deserialize, Serialize};

use crate::{commons::{id_vec::IdVec, rng::Rng}, resources::material::MaterialId};

use super::{creature::{CreatureId, Profession}, item::ItemId};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct SiteId(usize);
impl crate::commons::id_vec::Id for SiteId {
    fn new(id: usize) -> Self {
        SiteId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Sites = IdVec<Site>;

#[derive(Serialize, Deserialize)]
pub(crate) struct Site {
    pub(crate) xy: Vec2i,
    pub(crate) name: Option<String>,
    pub(crate) creatures: Vec<CreatureId>,
    pub(crate) cemetery: Vec<CreatureId>,
    pub(crate) resources: SiteResources,
    pub(crate) settlement: Option<SettlementComponent>,
    pub(crate) artifacts: Vec<ItemId>,
    pub(crate) population_peak: (i32, u32),
    pub(crate) site_type: SiteType,
    pub(crate) structures: Vec<Structure>,
}

impl Site {

    pub(crate) fn name(&self) -> &str {
        if let Some(name) = &self.name {
            return name.as_str();
        }
        match &self.site_type {
            SiteType::BanditCamp => "Bandit camp",
            SiteType::VarningrLair => "Varningr lair",
            SiteType::WolfPack => "Wolf den",
            SiteType::Village => "Village",
        }
    }

    pub(crate) fn remove_creature(&mut self, id: &CreatureId) {
        if let Some(idx) = self.creatures.iter().position(|another| another == id) {
            if let Some(structure) = self.structure_occupied_by_mut(&id) {
                structure.remove_ocuppant(id);
            }
            self.creatures.remove(idx);
        }
    }

    pub(crate) fn select_new_profession(&self, rng: &mut Rng) -> Profession {
        match self.site_type {
            SiteType::BanditCamp => Profession::Bandit,
            SiteType::Village => {
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
            SiteType::VarningrLair => Profession::Beast,
            SiteType::WolfPack => Profession::Beast,
        }
    }

    pub(crate) fn structure_occupied_by(&self, creature_id: &CreatureId) -> Option<&Structure> {
        return self.structures.iter().find(|structure| structure.occupants.binary_search(creature_id).is_ok());
    }

    pub(crate) fn structure_occupied_by_mut(&mut self, creature_id: &CreatureId) -> Option<&mut Structure> {
        return self.structures.iter_mut().find(|structure| structure.occupants.binary_search(creature_id).is_ok());
    }

}

#[derive(Serialize, Deserialize)]
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
mod tests_site {
    use crate::commons::id_vec::Id;
    use super::*;

    #[test]
    fn test_remove_creature() {
        let mut site = Site {
            xy: Vec2i(0, 0),
            creatures: Vec::new(),
            cemetery: Vec::new(),
            resources: SiteResources {
                food: 0.
            },
            name: None,
            settlement: None,
            artifacts: Vec::new(),
            population_peak: (0, 0),
            site_type: SiteType::Village,
            structures: Vec::new()
        };
        site.creatures.push(CreatureId::mock(0));
        site.creatures.push(CreatureId::mock(1));
        site.remove_creature(&CreatureId::mock(0));
        assert_eq!(site.creatures.len(), 1);
        assert_eq!(site.creatures[0].as_usize(), 1);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum SiteType {
    Village,
    BanditCamp,
    WolfPack,
    VarningrLair,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct SiteResources {
    // 1 unit = enough food for 1 adult for 1 year
    pub(crate) food: f32,
}

impl Add for SiteResources {
    type Output = SiteResources;

    fn add(self, other: SiteResources) -> SiteResources {
        return SiteResources {
            food: self.food + other.food
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Structure {
    structure_type: StructureType,
    status: StructureStatus,
    /// Occupants of this structure, ordered
    occupants: Vec<CreatureId>,
    pub(crate) generated_data: Option<StructureGeneratedData>,
}

impl Structure {

    pub(crate) fn new(structure_type: StructureType) -> Self {
        Self {
            structure_type,
            status: StructureStatus::Occupied,
            occupants: Vec::new(),
            generated_data: None
        }
    }

    pub(crate) fn get_status(&self) -> &StructureStatus {
        return &self.status;
    }

    pub(crate) fn get_type(&self) -> &StructureType {
        return &self.structure_type;
    }

    pub(crate) fn occupants(&self) -> impl Iterator<Item = &CreatureId> {
        return self.occupants.iter();
    }

    pub(crate) fn occupants_drain<F>(&mut self, predicate: F) -> Vec<CreatureId> where F: Fn(&CreatureId) -> bool {
        let mut vec: Vec<_> = Vec::new();
        for i in (0..self.occupants.len()).rev() {
            if predicate(&self.occupants[i]) {
                vec.push(self.occupants.remove(i));
            }
        }
        if self.occupants.len() == 0 {
            self.status = StructureStatus::Abandoned;
        }
        return vec;
    }

    pub(crate) fn occupants_take(&mut self) -> Vec<CreatureId> {
        self.status = StructureStatus::Abandoned;
        return self.occupants.drain(..).collect();
    }    

    pub(crate) fn occupant_count(&self) -> usize {
        return self.occupants.len();
    }

    pub(crate) fn add_ocuppant(&mut self, creature_id: CreatureId) {
        let pos = self.occupants.binary_search(&creature_id);
        match pos {
            Ok(_) => (),
            Err(pos) => self.occupants.insert(pos, creature_id),
        }
        self.status = StructureStatus::Occupied;
    }

    pub(crate) fn remove_ocuppant(&mut self, creature_id: &CreatureId) {
        if let Some(idx) = self.occupants.iter().position(|another| another == creature_id) {
            self.occupants.remove(idx);
        }
        if self.occupants.len() == 0 {
            self.status = StructureStatus::Abandoned;
        }
    } 

}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct StructureGeneratedData {
    pieces: Vec<(String, [u8; 4])>,
    pub(crate) last_generated_status: StructureStatus,
    pub(crate) spawn_points: Vec<Vec2i>
}

impl StructureGeneratedData {

    pub(crate) fn new(status: StructureStatus) -> Self {
        Self {
            pieces: Vec::new(),
            last_generated_status: status,
            spawn_points: Vec::new()
        }
    }

    pub(crate) fn add_piece(&mut self, piece_name: String, rect: [u8; 4]) {
        self.pieces.push((piece_name, rect));
    }

    pub(crate) fn pieces(&self) -> impl Iterator<Item = &(String, [u8; 4])> {
        self.pieces.iter()
    }

}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum StructureStatus {
    Occupied,
    // TODO(7gOA81VK): Abandoned since?
    Abandoned
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum StructureType {
    House,
    TownHall,
    BanditCamp
}