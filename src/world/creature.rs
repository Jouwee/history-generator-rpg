use crate::commons::id_vec::IdVec;

use super::{date::WorldDate, species::SpeciesId, unit::UnitResources, world::ArtifactId};


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub struct CreatureId(usize);
impl CreatureId {
    pub fn ancients() -> CreatureId {
        return CreatureId(0);
    }
}
impl crate::commons::id_vec::Id for CreatureId {
    fn new(id: usize) -> Self {
        CreatureId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub type Creatures = IdVec<Creature>;

#[derive(Clone)]
pub struct Creature {
    pub species: SpeciesId,
    pub birth: WorldDate,
    pub gender: CreatureGender,
    pub death: Option<(WorldDate, CauseOfDeath)>,
    pub profession: Profession,
    pub father: CreatureId,
    pub mother: CreatureId,
    pub spouse: Option<CreatureId>,
    pub offspring: Vec<CreatureId>,
    pub details: Option<CreatureDetails>
}

impl Creature {

    pub fn details(&mut self) -> &mut CreatureDetails {
        if self.details.is_none() {
            self.details = Some(CreatureDetails {
                inventory: Vec::new()
            })
        }
        return self.details.as_mut().expect("Already checked")
    }

}

#[derive(Clone)]
pub struct CreatureDetails {
    pub inventory: Vec<ArtifactId>
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CreatureGender {
    Male, Female
}

impl CreatureGender {

    pub fn is_male(&self) -> bool {
        if let CreatureGender::Male = self {
            return true
        }
        return false
    }

    pub fn is_female(&self) -> bool {
        if let CreatureGender::Female = self {
            return true
        }
        return false
    }

}

#[derive(Clone, Copy, Debug)]
pub enum CauseOfDeath {
    OldAge,
    Starvation,
    Disease,
}


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Profession {
    // Someone that doesn't work. Usually children and elders, but could be reserved for nitwits.
    None,
    // Workers
    // A peasant is someone trying to make the ends meet. Usually poor, they produce enough food to feed themselves and maybe a child, and pay a little in taxes.
    Peasant,
    Farmer,
    // Military
    Guard,
    // Artisans
    Blacksmith,
    Sculptor,
    // Political
    Ruler
}

impl Profession {

    pub fn base_resource_production(&self) -> UnitResources {
        match self {
            Profession::None => UnitResources { food: 0. },
            Profession::Peasant => UnitResources { food: 1.5 },
            Profession::Farmer => UnitResources { food: 3.0 },
            Profession::Guard => UnitResources { food: 0. },
            Profession::Blacksmith => UnitResources { food: 0. },
            Profession::Sculptor => UnitResources { food: 0. },
            Profession::Ruler => UnitResources { food: 0. },
        }
    }

    pub fn is_for_life(&self) -> bool {
        match self {
            Profession::None | Profession::Peasant | Profession::Farmer  | Profession::Guard | Profession::Blacksmith | Profession::Sculptor => false,
            Profession::Ruler => true,
        }
    }

}