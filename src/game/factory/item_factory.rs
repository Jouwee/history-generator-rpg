use crate::{commons::{rng::Rng, strings::Strings}, resources::item_blueprint::NameBlueprintComponent, world::item::{ArtworkScene, ItemMakeArguments, ItemQuality}, Item, Resources};

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
        }
    }

    pub(crate) fn weapon<'a>(rng: &'a mut Rng, resources: &'a Resources) -> WeaponFactory<'a> {
        return WeaponFactory { rng: rng, resources: resources, quality: None, named: false }
    }

    pub(crate) fn torso_garment<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let blueprint = resources.item_blueprints.find("itb:shirt");
         let item = blueprint.make(vec!(
        ), &resources);
        return item;
    }

    pub(crate) fn inner_armor<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let blueprint = resources.item_blueprints.find("itb:armor");
         let item = blueprint.make(vec!(
        ), &resources);
        return item;
    }

    pub(crate) fn pants<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let blueprint = resources.item_blueprints.find("itb:pants");
         let item = blueprint.make(vec!(
        ), &resources);
        return item;
    }

    pub(crate) fn boots<'a>(_rng: &'a mut Rng, resources: &'a Resources) -> Item {
        let blueprint = resources.item_blueprints.find("itb:boots");
         let item = blueprint.make(vec!(
        ), &resources);
        return item;
    }

    pub(crate) fn statue(rng: &mut Rng, resources: &Resources, scene: ArtworkScene) -> Item {
        let material_id = match rng.randu_range(0, 3) {
            0 => resources.materials.id_of("mat:steel"),
            1 => resources.materials.id_of("mat:copper"),
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

pub(crate) struct WeaponFactory<'a> {
    rng: &'a mut Rng,
    resources: &'a Resources,
    quality: Option<ItemQuality>,
    named: bool,
}

impl<'a> WeaponFactory<'a> {

    pub(crate) fn quality(mut self, quality: ItemQuality) -> Self {
        self.quality = Some(quality);
        return self
    }

    pub(crate) fn named(mut self) -> Self {
        self.named = true;
        return self
    }

    pub(crate) fn make(&mut self) -> Item {
        let material_id = match self.rng.randu_range(0, 2) {
            0 => self.resources.materials.id_of("mat:steel"),
            _ => self.resources.materials.id_of("mat:bronze")
        };

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
        let blueprint = match self.rng.randu_range(0, 2) {
            0 => self.resources.item_blueprints.find("itb:sword"),
            _ => self.resources.item_blueprints.find("itb:mace")
        };
        let handle = self.resources.materials.id_of("mat:oak");
        let pommel = self.resources.materials.id_of("mat:bronze");
        item = blueprint.make(vec!(
            ItemMakeArguments::PrimaryMaterial(material_id),
            ItemMakeArguments::SecondaryMaterial(handle),
            ItemMakeArguments::DetailsMaterial(pommel),
            ItemMakeArguments::Quality(quality),
        ), &self.resources);

        if self.named {
            if let Some(name_blueprint) = &blueprint.name_blueprint {
                item.special_name = Some(self.make_item_name(name_blueprint));
            }
        }

        return item;
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