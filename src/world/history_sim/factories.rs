// TODO: Break into files

use crate::{commons::{rng::Rng, strings::Strings}, resources::resources::Resources, world::{creature::{Creature, CreatureGender, CreatureId, Profession}, date::WorldDate, item::{ArtworkScene, Item, ItemQuality, Mace, Sword}, material::MaterialId, species::SpeciesId}};

use super::structs::World;

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
                birth: *now - WorldDate::new(age, 0, 0),
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
                birth: *now - WorldDate::new(age, 0, 0),
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
                birth: *now - WorldDate::new(age + self.rng.randi_range(-5, 5), 0 ,0),
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

pub struct ArtifactFactory {

}

impl ArtifactFactory {

    pub fn create_artifact(rng: &mut Rng, resources: &Resources, material_id: &MaterialId) -> Item {
        let material_id = material_id.clone();
        let item;
        match rng.randu_range(0, 2) {
            0 => {
                let mut blade = resources.materials.id_of("mat:steel");
                let mut handle = resources.materials.id_of("mat:oak");
                let mut guard = resources.materials.id_of("mat:bronze");
                let mut pommel = resources.materials.id_of("mat:bronze");
                match rng.randu_range(0, 4) {
                    1 => blade = material_id,
                    2 => guard = material_id,
                    3 => handle = material_id,
                    _ => pommel = material_id,
                }
                let mut sword = Sword::new(ItemQuality::Legendary, handle, blade, pommel, guard, &resources.materials);
                sword.name = Some(Self::artifact_name(rng.derive("name"), vec!(
                    "sword", "blade", "slash", "fang", "tongue", "kiss", "wing", "edge", "talon"
                )));
                item = Item::Sword(sword)
            },
            _ => {
                let mut head = resources.materials.id_of("mat:steel");
                let mut handle = resources.materials.id_of("mat:oak");
                let mut pommel = resources.materials.id_of("mat:bronze");
                match rng.randu_range(0, 3) {
                    1 => head = material_id,
                    2 => handle = material_id,
                    _ => pommel = material_id,
                }
                let mut mace = Mace::new(ItemQuality::Legendary, handle, head, pommel, &resources.materials);
                mace.name = Some(Self::artifact_name(rng.derive("name"), vec!(
                    "breaker", "kiss", "fist", "touch"
                )));
                item = Item::Mace(mace)
            }
        }
        return item;
    }

    pub fn create_statue(rng: &mut Rng, resources: &Resources, material_id: &MaterialId, subject: CreatureId, world: &World) -> Item {
        let material = resources.materials.id_of("mat:bronze");
        let creature = world.get_creature(&subject);
        if let Some(details) = &creature.details {
            if let Some(item) = details.inventory.first() {
                return Item::Statue { material: material, scene: ArtworkScene::FullBody { creature_id: subject, artifact_id: Some(*item) } }        
            }
        }

        return Item::Statue { material: material, scene: ArtworkScene::Bust { creature_id: subject } }
    }

    fn artifact_name(mut rng: Rng, suffixes: Vec<&str>) -> String {
        let preffixes = [
            "whisper", "storm", "fire", "moon", "sun", "ice", "raven", "thunder", "flame", "frost", "ember"
        ];
        let prefix = preffixes[rng.randu_range(0, preffixes.len())];
        let suffix = suffixes[rng.randu_range(0, suffixes.len())];
        return Strings::capitalize(format!("{prefix}{suffix}").as_str());
    }
}