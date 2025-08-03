use std::usize;

use crate::{commons::{bitmask::bitmask_get, id_vec::{Id, IdVec}, rng::Rng, strings::Strings}, resources::species::SpeciesId, world::plot::{PlotGoal, PlotId}, Resources};

use super::{date::WorldDate, item::ItemId, lineage::LineageId, unit::UnitResources, world::World};


#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub(crate) struct CreatureId(usize);
impl CreatureId {
    pub(crate) fn ancients() -> CreatureId {
        // TODO(esa51vK6): Workaround
        return CreatureId(usize::MAX);
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

pub(crate) const SIM_FLAG_INTELIGENT: u8 = 0b00000001;
pub(crate) const SIM_FLAG_GREAT_BEAST: u8 = 0b00000010;

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
    pub(crate) lineage: Option<LineageId>,
    pub(crate) experience: u32,
    pub(crate) details: Option<CreatureDetails>,
    pub(crate) sim_flags: u8,
    pub(crate) relationships: Vec<Relationship>,
    pub(crate) supports_plot: Option<PlotId>,
    // TODO(IhlgIYVA): Set maybe?
    pub(crate) goals: Vec<Goal>,
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

        if let Some(lineage) = &self.lineage {
            let lineage = world.lineages.get(lineage);
            let culture = resources.cultures.get(&lineage.culture);
            let name_model = match &self.gender {
                CreatureGender::Male => &culture.first_name_male_model,
                CreatureGender::Female => &culture.first_name_female_model,
            };
            let name = name_model.generate(&Rng::seeded(id.as_usize()), 5, 13);
            return format!("{} {}", Strings::capitalize(&name), Strings::capitalize(&lineage.name));
        }

        let species = resources.species.get(&self.species);
        return format!("the {}", species.name);
    }

    pub(crate) fn networth_range(&self) -> [i32; 2] {
        return self.profession.networth_range()
    }

    pub(crate) fn sim_flag_is_inteligent(&self) -> bool {
        return bitmask_get(self.sim_flags, SIM_FLAG_INTELIGENT)
    }

    pub(crate) fn sim_flag_is_great_beast(&self) -> bool {
        return bitmask_get(self.sim_flags, SIM_FLAG_GREAT_BEAST)
    }

    pub(crate) fn relationship_find_mut_or_insert(&mut self, self_creature_id: &CreatureId, other_creature_id: CreatureId, other_creature: &Creature) -> &mut Relationship {
        let pos = self.relationships.binary_search_by(|r| r.creature_id.cmp(&other_creature_id));
        match pos {
            Ok(pos) => self.relationships.get_mut(pos).expect("Just checked"),
            Err(pos) => {
                self.relationships.insert(pos, Relationship::new(self_creature_id, &self, other_creature_id, other_creature));
                self.relationships.get_mut(pos).expect("Just checked")
            }
        }
    }

    pub(crate) fn relationship_find(&self, other_creature_id: CreatureId) -> Option<&Relationship> {
        let pos = self.relationships.binary_search_by(|r| r.creature_id.cmp(&other_creature_id));
        match pos {
            Ok(pos) => Some(self.relationships.get(pos).expect("Just checked")),
            Err(_) => None
        }
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
    KilledInBattle(CreatureId)
}


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub(crate) enum Profession {
    // Someone that doesn't work. Usually children and elders, but could be reserved for nitwits.
    None,
    Beast,
    // Outlaws
    Bandit,
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
            Profession::Bandit => UnitResources { food: 0.8 },
            Profession::Guard => UnitResources { food: 0. },
            Profession::Blacksmith => UnitResources { food: 0. },
            Profession::Sculptor => UnitResources { food: 0. },
            Profession::Ruler => UnitResources { food: 0. },
            Profession::Beast => UnitResources { food: 1.5 },
        }
    }

    pub(crate) fn is_for_life(&self) -> bool {
        match self {
            Profession::None | Profession::Peasant | Profession::Farmer  | Profession::Guard | Profession::Blacksmith | Profession::Sculptor => false,
            Profession::Bandit => true,
            Profession::Ruler => true,
            Profession::Beast => true,
        }
    }

    pub(crate) fn networth_range(&self) -> [i32; 2] {
        match self {
            Profession::None => [0, 0],
            Profession::Peasant => [1, 10],
            Profession::Farmer => [3, 15],
            Profession::Bandit => [3, 15],
            Profession::Guard => [5, 20],
            Profession::Blacksmith =>  [5, 20],
            Profession::Sculptor =>  [5, 20],
            Profession::Ruler =>  [50, 100],
            Profession::Beast => [0, 0],
        }
    }

}

#[derive(Clone, Debug)]
/// A structure representing the relationship between a creature that holds this creature, and another creature
pub(crate) struct Relationship {
    /// Who the relationship is with
    pub(crate) creature_id: CreatureId,
    /// A value between -100 and 100 representing the general opinion that this creature holds with the other
    opinion: i8
}

impl Relationship {

    pub(crate) fn new(creature_id: &CreatureId, creature: &Creature, other_creature_id: CreatureId, other_creature: &Creature) -> Self {
        let mut opinion = 0;

        // My child
        if other_creature.father == *creature_id || other_creature.mother == *creature_id {
            opinion = 75;
        }

        // My parent
        if creature.father == other_creature_id || creature.mother == other_creature_id {
            opinion = 50;
        }

        // My sibling or half-sibling
        if (creature.father == other_creature.father && creature.father != CreatureId::ancients()) || 
            (creature.mother == other_creature.mother && creature.mother != CreatureId::ancients()) {
            opinion = 50;
        }

        Relationship {
            creature_id: other_creature_id,
            opinion
        }
    }

    pub(crate) fn add_opinion(&mut self, opinion: i8) {
        self.opinion = (self.opinion.saturating_add(opinion)).clamp(-100, 100);
    }

    pub(crate) fn mortal_enemy_or_worse(&self) -> bool {
        return self.opinion <= -75;
    }
    pub(crate) fn rival_or_worse(&self) -> bool {
        return self.opinion <= -20;
    }

    pub(crate) fn friend_or_better(&self) -> bool {
        return self.opinion >= 20;
    }

}

#[cfg(test)]
mod tests_relationship {
    use crate::world::world::fixture::WorldFixture;

    use super::*;

    #[test]
    fn test_init() {
        let mut world = WorldFixture::new();

        world.creature_a3_mut().father = world.creature_a1;
        world.creature_a3_mut().mother = world.creature_a2;
        world.creature_a4_mut().father = world.creature_a1;
        world.creature_a4_mut().mother = world.creature_a2;

        // Unknown
        let relationship = Relationship::new(&world.creature_a1, &world.creature_a1(), world.creature_a2, &world.creature_a2());
        assert_eq!(relationship.opinion, 0);

        // Father > Child
        let relationship = Relationship::new(&world.creature_a1, &world.creature_a1(), world.creature_a3, &world.creature_a3());
        assert_eq!(relationship.opinion, 75);

        // Child > Parent
        let relationship = Relationship::new(&world.creature_a3, &world.creature_a3(), world.creature_a1, &world.creature_a1());
        assert_eq!(relationship.opinion, 50);

        // Siblings
        let relationship = Relationship::new(&world.creature_a3, &world.creature_a3(), world.creature_a4, &world.creature_a4());
        assert_eq!(relationship.opinion, 50);


    }

    #[test]
    fn test_add_opiinion() {
        let world = WorldFixture::new();
        let mut relationship = Relationship::new(&world.creature_a1, &world.creature_a1(), world.creature_a2, &world.creature_a2());

        // Shouldn't overflow
        relationship.add_opinion(75);
        assert_eq!(relationship.opinion, 75);
        relationship.add_opinion(75);
        assert_eq!(relationship.opinion, 100);
        relationship.add_opinion(75);
        assert_eq!(relationship.opinion, 100);

        let mut relationship = Relationship::new(&world.creature_a1, &world.creature_a1(), world.creature_a2, &world.creature_a2());
        relationship.add_opinion(-75);
        assert_eq!(relationship.opinion, -75);
        relationship.add_opinion(-75);
        assert_eq!(relationship.opinion, -100);
        relationship.add_opinion(-75);
        assert_eq!(relationship.opinion, -100);
    }

}

#[derive(Clone, Debug)]
pub(crate) enum Goal {
    /// Wants a creature dead, by any means necessary
    KillBeast(CreatureId)
}

impl Goal {

    pub(crate) fn as_plot_goal(&self) -> Option<PlotGoal> {
        match self {
            Self::KillBeast(id) => Some(PlotGoal::KillBeast(*id))
        }
    }


    pub(crate) fn check_completed(&self, world: &World) -> bool {
        match self {
            Self::KillBeast(id) => world.creatures.get(id).death.is_some()
        }
    }

}