use crate::{commons::{id_vec::{Id, IdVec}, rng::Rng}, resources::species::SpeciesId, Resources};

use super::{date::WorldDate, item::ItemId, lineage::LineageId, unit::UnitResources, world::World};


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct CreatureId(usize);
impl CreatureId {
    pub(crate) fn ancients() -> CreatureId {
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

pub(crate) type Creatures = IdVec<Creature>;

#[derive(Clone)]
pub(crate) struct Creature {
    pub(crate) species: SpeciesId,
    pub(crate) birth: WorldDate,
    pub(crate) gender: CreatureGender,
    pub(crate) death: Option<(WorldDate, CauseOfDeath)>,
    pub(crate) profession: Profession,
    pub(crate) father: CreatureId,
    pub(crate) mother: CreatureId,
    pub(crate) spouse: Option<CreatureId>,
    pub(crate) offspring: Vec<CreatureId>,
    pub(crate) lineage: LineageId,
    pub(crate) details: Option<CreatureDetails>
}

impl Creature {

    pub(crate) fn details(&mut self) -> &mut CreatureDetails {
        if self.details.is_none() {
            self.details = Some(CreatureDetails {
                inventory: Vec::new()
            })
        }
        return self.details.as_mut().expect("Already checked")
    }

    pub(crate) fn name(&self, id: &CreatureId, world: &World, resources: &Resources) -> String {
        let lineage = world.lineages.get(&self.lineage);
        let culture = resources.cultures.get(&lineage.culture);
        let name_model = match &self.gender {
            CreatureGender::Male => &culture.first_name_male_model,
            CreatureGender::Female => &culture.first_name_female_model,
        };
        let name = name_model.generate(&Rng::seeded(id.as_usize()), 5, 13);
        return format!("{} {}", name, lineage.name)
    }

}

#[derive(Clone)]
pub(crate) struct CreatureDetails {
    pub(crate) inventory: Vec<ItemId>
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum CreatureGender {
    Male, Female
}

impl CreatureGender {

    pub(crate) fn is_male(&self) -> bool {
        if let CreatureGender::Male = self {
            return true
        }
        return false
    }

    pub(crate) fn is_female(&self) -> bool {
        if let CreatureGender::Female = self {
            return true
        }
        return false
    }

}

#[derive(Clone, Copy, Debug)]
pub(crate) enum CauseOfDeath {
    OldAge,
    Starvation,
    Disease,
}


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum Profession {
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

    pub(crate) fn base_resource_production(&self) -> UnitResources {
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

    pub(crate) fn is_for_life(&self) -> bool {
        match self {
            Profession::None | Profession::Peasant | Profession::Farmer  | Profession::Guard | Profession::Blacksmith | Profession::Sculptor => false,
            Profession::Ruler => true,
        }
    }

}