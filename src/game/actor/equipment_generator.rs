use crate::{game::inventory::inventory::Inventory, world::{creature::{Creature, CreatureId, Profession}, world::World}, ItemFactory, Resources, Rng};

pub(crate) struct EquipmentGenerator {

}

impl EquipmentGenerator {

    pub(crate) fn generate(creature_id: &CreatureId, rng: &mut Rng, world: &World, resources: &Resources) -> Inventory {
        let creature = world.creatures.get(creature_id);
        let mut inventory = Inventory::new();

        // TODO:
        let budget_range = creature.networth_range();
        let mut _budget = rng.randi_range(budget_range[0], budget_range[1]);

        let _ = inventory.add(ItemFactory::torso_garment(rng, &resources));
        let _ = inventory.add(ItemFactory::boots(rng, &resources));
        let _ = inventory.add(ItemFactory::pants(rng, &resources));

        if creature.profession == Profession::Guard || creature.profession == Profession::Bandit || creature.profession == Profession::Ruler {
            let item = ItemFactory::weapon(rng, &resources).make();
            let _ = inventory.add(item);
            let _ = inventory.add(ItemFactory::inner_armor(rng, &resources));
        }

        Self::add_artifacts(&mut inventory, &creature, world);

        inventory.auto_equip();

        return inventory
    }

    fn add_artifacts(inventory: &mut Inventory, creature: &Creature, world: &World) {
        if let Some(details) = &creature.details {
            for id in details.inventory.iter() {
                let item = world.artifacts.get(id);
                let _ = inventory.add(item.clone());
            }
        }
    }

}