use crate::{commons::{bitmask::bitmask_get, rng::Rng, strings::Strings}, resources::{item_blueprint::NameBlueprintComponent, material::MaterialId}, world::item::{ArtworkScene, ItemMakeArguments, ItemQuality}, Item, Resources};

pub(crate) struct ItemFactory {}

impl ItemFactory {

    #[cfg(test)]
    pub(crate) fn test() -> Item {
        use image::ImageReader;
        use crate::engine::pallete_sprite::PalleteSprite;
        let image = ImageReader::open("./assets/sprites/missing.png").unwrap().decode().unwrap();
        return Item {
            action_provider: None,
            artwork_scene: None,
            equippable: None,
            material: None,
            mellee_damage: None,
            armor: None,
            name: String::from(""),
            placed_sprite: PalleteSprite::new(image),
            quality: None,
            special_name: None,
            owner: None,
        }
    }

    pub(crate) fn quest_reward(resources: &Resources) -> Item {
        let mut rng = Rng::rand();
        match rng.randu_range(0, 6) {
            0 | 1 => Self::inner_armor(&mut rng, resources),
            2 | 3 => Self::spell_tome(&mut rng, resources),
            4 | 5 | _ => Self::weapon(&mut rng, resources).make(),
        }
    }

    pub(crate) fn weapon<'a>(rng: &'a mut Rng, resources: &'a Resources) -> WeaponFactory<'a> {
        return WeaponFactory { rng: rng, resources: resources, quality: None, material_pool: None, named: false }
    }

    pub(crate) fn spell_tome<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let mut rng = Rng::rand();
        let blueprint = match rng.randu_range(0, 4) {
            1 => resources.item_blueprints.find("itb:tome_fireball"),
            2 => resources.item_blueprints.find("itb:tome_firebolt"),
            3 => resources.item_blueprints.find("itb:tome_teleport"),
            _ => resources.item_blueprints.find("itb:tome_rockpillar"),
        };
        let item = blueprint.make(vec!(), &resources);
        return item;
    }

    pub(crate) fn torso_garment<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let blueprint = resources.item_blueprints.find("itb:shirt");
        let item = blueprint.make(vec!(), &resources);
        return item;
    }

    pub(crate) fn crown<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let blueprint = resources.item_blueprints.find("itb:crown");
        let item = blueprint.make(vec!(), &resources);
        return item;
    }

    pub(crate) fn inner_armor<'a>(rng: &'a mut Rng, resources: &'a Resources) -> Item {
        if rng.rand_chance(0.5) {
            let blueprint = resources.item_blueprints.find("itb:brigandine");
            let item = blueprint.make(vec!(), &resources);
            return item;
        } else {
            let blueprint = resources.item_blueprints.find("itb:cuirass");
            let material_id = match rng.randu_range(0, 4) {
                0 => resources.materials.id_of("mat:steel"),
                1 => resources.materials.id_of("mat:iron"),
                2 => resources.materials.id_of("mat:copper"),
                _ => resources.materials.id_of("mat:bronze")
            };
            let item = blueprint.make(vec!(ItemMakeArguments::PrimaryMaterial(material_id)), &resources);
            return item;
        }
    }

    pub(crate) fn pants<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let blueprint = resources.item_blueprints.find("itb:pants");
        let item = blueprint.make(vec!(), &resources);
        return item;
    }

    pub(crate) fn boots<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let blueprint = resources.item_blueprints.find("itb:boots");
        let item = blueprint.make(vec!(), &resources);
        return item;
    }

    pub(crate) fn statue(rng: &mut Rng, resources: &Resources, scene: ArtworkScene) -> Item {
        let material_id = match rng.randu_range(0, 4) {
            0 => resources.materials.id_of("mat:steel"),
            1 => resources.materials.id_of("mat:iron"),
            2 => resources.materials.id_of("mat:copper"),
            _ => resources.materials.id_of("mat:bronze")
        };

        let blueprint = resources.item_blueprints.find("itb:statue");

        let item = blueprint.make(vec!(
            ItemMakeArguments::PrimaryMaterial(material_id),
            ItemMakeArguments::Scene(scene),
        ), &resources);
        return item;
    }

}

type MaterialPool = Vec<(MaterialId, usize)>;

pub(crate) struct WeaponFactory<'a> {
    rng: &'a mut Rng,
    resources: &'a Resources,
    quality: Option<ItemQuality>,
    material_pool: Option<&'a mut MaterialPool>,
    named: bool,
}

impl<'a> WeaponFactory<'a> {

    pub(crate) fn quality(mut self, quality: ItemQuality) -> Self {
        self.quality = Some(quality);
        return self
    }


    pub(crate) fn material_pool(mut self, material_pool: Option<&'a mut MaterialPool>) -> Self {
        self.material_pool = material_pool;
        return self
    }

    pub(crate) fn named(mut self) -> Self {
        self.named = true;
        return self
    }

    pub(crate) fn make(&mut self) -> Item {
        let quality = match self.quality {
            Some(quality) => quality,
            None => {
                let f_quality = self.rng.randf();
                if f_quality < 0.5 {
                    ItemQuality::Poor
                } else if f_quality < 0.9 {
                    ItemQuality::Normal
                } else if f_quality < 0.99 {
                    ItemQuality::Good
                } else {
                    ItemQuality::Excelent
                }
            }
        };

        let mut item;
        let blueprint = match self.rng.randu_range(0, 3) {
            0 => self.resources.item_blueprints.find("itb:sword"),
            1 => self.resources.item_blueprints.find("itb:mace"),
            _ => self.resources.item_blueprints.find("itb:axe"),
        };

        let mut arguments = vec!(ItemMakeArguments::Quality(quality));

        if let Some(material_blueprint) = &blueprint.material {
            let always_available = vec!(
                self.resources.materials.id_of("mat:oak"),
                self.resources.materials.id_of("mat:birch"),
                self.resources.materials.id_of("mat:copper"),
                self.resources.materials.id_of("mat:bronze"),
                self.resources.materials.id_of("mat:iron"),
                self.resources.materials.id_of("mat:steel"),
            );

            let primary = self.pick_material(material_blueprint.primary_tag_bitmask, &always_available);
            arguments.push(ItemMakeArguments::PrimaryMaterial(primary));

            if let Some(secondary_bitmask) = material_blueprint.secondary_tag_bitmask {
                let secondary = self.pick_material(secondary_bitmask, &always_available);
                arguments.push(ItemMakeArguments::SecondaryMaterial(secondary));
            }

            if let Some(details_bitmask) = material_blueprint.details_tag_bitmask {
                let details = self.pick_material(details_bitmask, &always_available);
                arguments.push(ItemMakeArguments::DetailsMaterial(details));
            }
        }

        item = blueprint.make(arguments, &self.resources);

        if self.named {
            if let Some(name_blueprint) = &blueprint.name_blueprint {
                item.special_name = Some(self.make_item_name(name_blueprint));
            }
        }

        return item;
    }

    fn pick_material(&mut self, mat_tag_bitmask: u8, always_available_materials: &Vec<MaterialId>) -> MaterialId {

        enum MaterialSource {
            Pool(MaterialId),
            AlwaysAvailable(MaterialId),
        }
        let mut candidates = Vec::new();

        if let Some(material_pool) = &self.material_pool {
            for (material_id, _count) in material_pool.iter() {
                let material = self.resources.materials.get(material_id);
                if bitmask_get(material.tags_bitmask, mat_tag_bitmask) {
                    candidates.push(MaterialSource::Pool(*material_id));
                }
            };
        }

        for material_id in always_available_materials.iter() {
            let material = self.resources.materials.get(material_id);
            if bitmask_get(material.tags_bitmask, mat_tag_bitmask) {
                candidates.push(MaterialSource::AlwaysAvailable(*material_id));
            }
        };

        let selected_material = self.rng.item(&candidates);
        match selected_material {
            Some(MaterialSource::Pool(id)) => {
                Self::consume_material(self.material_pool.as_mut().expect("Checked above"), id).expect("I don't see how this would happen");
                *id
            },
            Some(MaterialSource::AlwaysAvailable(id)) => *id,
            _ => panic!("No materials available to create item"),
        }
    }

    fn consume_material(pool: &mut MaterialPool, id: &MaterialId) -> Result<(), ()> {
        let position = pool.iter().position(|(l_id, _count)| l_id == id);
        if let Some(position) = position {
            pool[position].1 -= 1;
            if pool[position].1 <= 0 {
                pool.remove(position);
            }
            return Ok(())
        }
        return Err(())
    }

    fn make_item_name(&mut self, blueprint: &NameBlueprintComponent) -> String {
        let preffixes = [
            "whisper", "storm", "fire", "moon", "sun", "ice", "raven", "thunder", "flame", "frost", "ember"
        ];
        let prefix = preffixes[self.rng.randu_range(0, preffixes.len())];
        let suffix = self.rng.item(&blueprint.suffixes).expect("Namable items should have suffixes");
        return Strings::capitalize(format!("{prefix}{suffix}").as_str());
    }

}