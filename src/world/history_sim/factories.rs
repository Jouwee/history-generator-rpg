// TODO: Break into files

use crate::{commons::rng::Rng, world::species::SpeciesId};

use super::structs::{Creature, CreatureGender, CreatureId, Profession, World, WorldDate};

pub struct CreatureFactory {
    rng: Rng
}

impl CreatureFactory {

    pub fn new(rng: Rng) -> CreatureFactory {
        CreatureFactory { rng }
    }

    pub fn make_family_or_single(&mut self, now: &WorldDate, species: SpeciesId, world: &mut World) -> Vec<CreatureId> {
        let age = self.rng.randi_range(20, 50);
        // Single
        if self.rng.rand_chance(0.5) {
            let mut gender = CreatureGender::Male;
            if self.rng.rand_chance(0.5) {
                gender = CreatureGender::Female;
            }
            let creature_id = world.add_creature(Creature {
                birth: now.subtract(&WorldDate::year(age)),
                death: None,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                profession: Profession::Peasant,
                spouse: None,
                gender,
                offspring: Vec::new(),
                species: species,
                details: None,
            });
            return vec!(creature_id)
        } else {
            let mut family = Vec::new();

            // TODO: Children

            let father_id = world.add_creature(Creature {
                birth: now.subtract(&WorldDate::year(age)),
                death: None,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                profession: Profession::Peasant,
                // TODO:
                spouse: None,
                gender: CreatureGender::Male,
                offspring: Vec::new(),
                species: species,
                details: None,
            });
            family.push(father_id);
            
            let mother_id = world.add_creature(Creature {
                birth: now.subtract(&WorldDate::year(age + self.rng.randi_range(-5, 5))),
                death: None,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                profession: Profession::Peasant,
                spouse: Some(father_id),
                gender: CreatureGender::Female,
                offspring: Vec::new(),
                species: species,
                details: None,
            });
            family.push(mother_id);


            return family;
        }
    }
}