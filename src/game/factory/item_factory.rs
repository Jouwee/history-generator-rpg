use crate::{commons::rng::Rng, world::item::ItemQuality, Item, Mace, Resources, Sword};

pub(crate) struct ItemFactory {}

impl ItemFactory {

    pub(crate) fn weapon(rng: &mut Rng, resources: &Resources) -> Item {
        let material_id = match rng.randu_range(0, 2) {
            0 => resources.materials.id_of("mat:steel"),
            _ => resources.materials.id_of("mat:bronze")
        };
        let f_quality = rng.randf();
        let quality;
        if f_quality < 0.5 {
            quality = ItemQuality::Poor;
        } else if f_quality < 0.9 {
            quality = ItemQuality::Normal;
        } else if f_quality < 0.99 {
            quality = ItemQuality::Good;
        } else {
            quality = ItemQuality::Excelent;
        }

        let item;
        match rng.randu_range(0, 2) {
            0 => {
                let blade = material_id;
                let handle = resources.materials.id_of("mat:oak");
                let guard = resources.materials.id_of("mat:bronze");
                let pommel = resources.materials.id_of("mat:bronze");
                let sword = Sword::new(quality, handle, blade, pommel, guard, &resources.materials);
                item = Item::Sword(sword)
            },
            _ => {
                let head = material_id;
                let handle = resources.materials.id_of("mat:oak");
                let pommel = resources.materials.id_of("mat:bronze");
                let mace = Mace::new(quality, handle, head, pommel, &resources.materials);
                item = Item::Mace(mace)
            }
        }
        return item;
    }

}