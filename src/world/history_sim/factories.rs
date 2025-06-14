use crate::{commons::{rng::Rng, xp_table::level_to_xp}, resources::{resources::Resources, species::SpeciesId}, world::{creature::{Creature, CreatureGender, CreatureId, Profession, SIM_FLAG_INTELIGENT}, date::WorldDate, item::{ArtworkScene, Item, ItemQuality}, lineage::Lineage, world::World}, ItemFactory};

pub(crate) struct CreatureFactory {
    rng: Rng
}

impl CreatureFactory {

    pub(crate) fn new(rng: Rng) -> CreatureFactory {
        CreatureFactory { rng }
    }

    pub(crate) fn make_family_or_single(&mut self, now: &WorldDate, species: SpeciesId, world: &mut World, resources: &Resources) -> Vec<CreatureId> {
        let age = self.rng.randi_range(20, 50);

        let culture_id = resources.cultures.random();
        let culture = resources.cultures.get(&culture_id);
        let lineage = world.lineages.add(Lineage::new(resources.cultures.random(), &culture));

        // Single
        if self.rng.rand_chance(0.5) {
            let mut gender = CreatureGender::Male;
            if self.rng.rand_chance(0.5) {
                gender = CreatureGender::Female;
            }
            let creature_id = world.creatures.add(Creature {
                birth: *now - WorldDate::new(age, 0, 0),
                death: None,
                lineage,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                profession: Profession::Peasant,
                spouse: None,
                gender,
                offspring: Vec::new(),
                species: species,
                experience: 0,
                details: None,
                sim_flags: SIM_FLAG_INTELIGENT
            });
            return vec!(creature_id)
        } else {
            let mut family = Vec::new();

            // TODO: Children

            let father_id = world.creatures.add(Creature {
                birth: *now - WorldDate::new(age, 0, 0),
                death: None,
                lineage,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                profession: Profession::Peasant,
                // TODO:
                spouse: None,
                gender: CreatureGender::Male,
                offspring: Vec::new(),
                species: species,
                experience: 0,
                details: None,
                sim_flags: SIM_FLAG_INTELIGENT
            });
            family.push(father_id);
            
            let mother_id = world.creatures.add(Creature {
                birth: *now - WorldDate::new(age + self.rng.randi_range(-5, 5), 0 ,0),
                death: None,
                lineage,
                father: CreatureId::ancients(),
                mother: CreatureId::ancients(),
                profession: Profession::Peasant,
                spouse: Some(father_id),
                gender: CreatureGender::Female,
                offspring: Vec::new(),
                species: species,
                experience: 0,
                details: None,
                sim_flags: SIM_FLAG_INTELIGENT
            });
            family.push(mother_id);


            return family;
        }
    }

    pub(crate) fn make_single(&mut self, species: SpeciesId, level: u16, sim_flags: u8, world: &mut World, resources: &Resources) -> CreatureId {
        
        // TODO(PaZs1uBR): A beast doesn't have a lineage
        let culture_id = resources.cultures.random();
        let culture = resources.cultures.get(&culture_id);
        let lineage = world.lineages.add(Lineage::new(resources.cultures.random(), &culture));

        let mut gender = CreatureGender::Male;
        if self.rng.rand_chance(0.5) {
            gender = CreatureGender::Female;
        }
        let creature_id = world.creatures.add(Creature {
            birth: world.date.clone(),
            death: None,
            lineage,
            father: CreatureId::ancients(),
            mother: CreatureId::ancients(),
            // TODO(PaZs1uBR): A creature doesn't have a profession
            profession: Profession::Peasant,
            spouse: None,
            experience: level_to_xp(level),
            gender,
            offspring: Vec::new(),
            species: species,
            details: None,
            sim_flags
        });
        return creature_id
    }
}

pub(crate) struct ArtifactFactory {

}

impl ArtifactFactory {

    pub(crate) fn create_artifact(rng: &mut Rng, resources: &Resources) -> Item {
        let item = ItemFactory::weapon(rng, resources)
            .quality(ItemQuality::Legendary)
            .named()
            .make();
        return item;
    }

    pub(crate) fn create_statue(resources: &Resources, subject: CreatureId, world: &World) -> Item {
        // TODO: Determinate
        let mut rng = Rng::rand();
        let creature = world.creatures.get(&subject);
        if let Some(details) = &creature.details {
            if let Some(item) = details.inventory.first() {
                return ItemFactory::statue(&mut rng, resources, ArtworkScene::FullBody { creature_id: subject, artifact_id: Some(*item) })
            }
        }

        return ItemFactory::statue(&mut rng, resources, ArtworkScene::Bust { creature_id: subject });
    }
}