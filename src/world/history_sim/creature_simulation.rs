use crate::{commons::rng::Rng, world::{creature::{CauseOfDeath, Creature, CreatureGender, CreatureId, Profession}, date::WorldDate, unit::Unit, world::World}};

pub(crate) struct CreatureSimulation {}

pub(crate) enum CreatureSideEffect {
    None,
    Death(CauseOfDeath),
    HaveChild,
    LookForMarriage,
    LookForNewJob,
    MakeArtifact,
    ComissionArtifact,
    ArtisanLookingForComission,
    BecomeBandit,
    AttackNearbyUnits,
}

const YEARLY_CHANCE_MARRY: f32 = 0.4;
const YEARLY_CHANCE_CHILD_MULT: f32 = 1.0;
const CHANCE_TO_STARVE: f32 = 0.2;
const BASE_DISEASE_CHANCE: f32 = 0.0015;
const CHANCE_NEW_JOB: f32 = 0.005;
const CHANCE_MAKE_INSPIRED_ARTIFACT: f32 = 0.005;
const CHANCE_TO_COMISSION_ARTIFACT_ON_BDAY: f32 = 0.5;
const CHANCE_TO_BECOME_BANDIT: f32 = 0.005;

impl CreatureSimulation {
    // TODO: Smaller steps
    pub(crate) fn simulate_step_creature(_step: &WorldDate, now: &WorldDate, rng: &mut Rng, unit: &Unit, creature: &Creature) -> CreatureSideEffect {
        let age = (*now - creature.birth).year();
        // Death by starvation
        if unit.resources.food <= 0. && rng.rand_chance(CHANCE_TO_STARVE) {
            return CreatureSideEffect::Death(CauseOfDeath::Starvation);
        }
        // Death by disease
        if rng.rand_chance(Self::chance_of_disease(now, &creature)) {
            return CreatureSideEffect::Death(CauseOfDeath::Disease);
        }

        if creature.sim_flag_is_great_beast() {
            // TODO(PaZs1uBR): Balance
            if rng.rand_chance(0.8) {
                return CreatureSideEffect::AttackNearbyUnits;
            }
        }

        // Get a profession
        if age >= 14 && creature.profession == Profession::None {
            return CreatureSideEffect::LookForNewJob;
        }
        if age >= 18 {
            // Have child
            if creature.gender.is_female() && creature.spouse.is_some()  {
                if rng.rand_chance(Self::chance_of_child(now, creature, unit.resources.food, unit.creatures.len())) {
                    return CreatureSideEffect::HaveChild;
                }
            }
            // Find a spouse
            if creature.spouse.is_none() {
                if rng.rand_chance(YEARLY_CHANCE_MARRY) {
                    return CreatureSideEffect::LookForMarriage;
                }
            }
            if age >= 60 {
                // Death of old age
                if rng.rand_chance(Self::chance_of_death_by_old_age(age as f32)) {
                    return CreatureSideEffect::Death(CauseOfDeath::OldAge);
                }
            }
            // Look for new job
            if !creature.profession.is_for_life() {
                if rng.rand_chance(CHANCE_NEW_JOB) {
                    return CreatureSideEffect::LookForNewJob;
                }
                if rng.rand_chance(CHANCE_TO_BECOME_BANDIT) {
                    return CreatureSideEffect::BecomeBandit;
                }
            }
            
            if creature.profession == Profession::Ruler && age % 10 == 0 {
                if rng.rand_chance(CHANCE_TO_COMISSION_ARTIFACT_ON_BDAY) {
                    return CreatureSideEffect::ComissionArtifact;
                }
            }

        }

        match creature.profession {
            Profession::Blacksmith => {
                if rng.rand_chance(CHANCE_MAKE_INSPIRED_ARTIFACT) {
                    return CreatureSideEffect::MakeArtifact;
                }
                return CreatureSideEffect::ArtisanLookingForComission;
            },
            Profession::Sculptor => {
                return CreatureSideEffect::ArtisanLookingForComission;
            },
            _ => ()
        }
        return CreatureSideEffect::None
    }

    fn chance_of_child(now: &WorldDate, creature: &Creature, unit_food_stock: f32, unit_population: usize) -> f32 {
        let food_excess_pct = unit_food_stock / unit_population as f32;
        let food_mult = (food_excess_pct - 1.).clamp(0.02, 1.);
        
        let children_mult = 1. - (creature.offspring.len() as f32 / 10.);
        let age = (*now - creature.birth).year() as f32;
        
        let fertility_mult = (0.96 as f32).powf(age - 18.) * (0.92 as f32).powf(age - 18.);

        return YEARLY_CHANCE_CHILD_MULT * fertility_mult * food_mult * children_mult;
    }

    fn chance_of_disease(now: &WorldDate, creature: &Creature) -> f32 {
        let age = (*now - creature.birth).year() as f32;
        // Children are more suceptible to disease
        if age < 18. {
            let boost = (age / 18.).powf(2.) + 1.;
            return BASE_DISEASE_CHANCE + (boost * BASE_DISEASE_CHANCE);
        }
        // Same as older people
        if age >= 40. {
            let boost = ((age - 40.) / 40.).powf(2.);
            return BASE_DISEASE_CHANCE + (boost * BASE_DISEASE_CHANCE);
        }
        return BASE_DISEASE_CHANCE;
    }

    fn chance_of_death_by_old_age(age: f32) -> f32 {
        return ((age - 60.) / 60.).powf(4.0).clamp(0., 1.)
    }

    pub(crate) fn have_child_with_spouse(now: &WorldDate, world: &World, rng: &mut Rng, creature_id: &CreatureId, creature: &mut Creature) -> Option<Creature> {
        let father_id = creature.spouse;
        if let Some(father_id) = father_id {
            let father = world.creatures.get(&father_id);
            let lineage = father.lineage.clone();
            let mut gender = CreatureGender::Male;
            if rng.rand_chance(0.5) {
                gender = CreatureGender::Female;
            }
            let child = Creature {
                birth: now.clone(),
                death: None,
                profession: Profession::None,
                lineage,
                mother: *creature_id,
                father: father_id,
                gender,
                offspring: Vec::new(),
                species: creature.species,
                spouse: None,
                details: None,
                experience: 0,
                sim_flags: father.sim_flags
            };
            return Some(child)
        }
        return None
    }

}
