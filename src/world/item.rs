use std::{cell::RefCell, collections::HashMap};

use image::{DynamicImage, RgbaImage};
use opengl_graphics::{Filter, Texture, TextureSettings};
use serde::{Deserialize, Serialize};

use crate::{commons::{damage_model::{DamageModel, DamageRoll}, id_vec::IdVec, strings::Strings}, engine::{gui::tooltip::{Tooltip, TooltipLine}, pallete_sprite::ColorMap}, game::actor::health_component::BodyPart, resources::{action::ActionId, item_blueprint::ItemBlueprintId, material::{MaterialId, Materials}, resources::Resources, species::SPECIES_SPRITE_SIZE}, Color};

use super::creature::CreatureId;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct ItemId(usize);
impl crate::commons::id_vec::Id for ItemId {
    fn new(id: usize) -> Self {
        Self(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Items = IdVec<Item>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Item {
    pub(crate) blueprint_id: ItemBlueprintId,
    pub(crate) name: String,
    pub(crate) special_name: Option<String>,
    pub(crate) owner: Option<CreatureId>,
    pub(crate) action_provider: Option<ActionProviderComponent>,
    pub(crate) material: Option<MaterialComponent>,
    pub(crate) quality: Option<QualityComponent>,
    pub(crate) mellee_damage: Option<MelleeDamageComponent>,
    pub(crate) armor: Option<ArmorComponent>,
    pub(crate) artwork_scene: Option<ArtworkSceneComponent>,
    // In-memory cache, shouldn't save
    #[serde(skip)] 
    pub(crate) cached_placed_texture: RefCell<Option<RgbaImage>>,
    #[serde(skip)]
    pub(crate) cached_inventory_texture: RefCell<Option<RgbaImage>>,
}

impl Item {

    pub(crate) fn name(&self, materials: &Materials) -> String {
        if let Some(name) = &self.special_name {
            return name.clone();
        }
        let mut name = self.name.clone();
        if let Some(material) = &self.material {
            let material = materials.get(&material.primary);
            name = format!("{} {name}", material.name)
        }
        if let Some(quality) = &self.quality {
            name = format!("{:?} {name}", quality.quality)
        }
        return name
    }

    pub(crate) fn make_tooltip(&self, materials: &Materials) -> Tooltip {
        let mut tooltip = Tooltip::new(&Strings::capitalize(&self.name(materials)));
        if let Some(_) = self.mellee_damage {
            tooltip.add_line(TooltipLine::DamageRoll(self.total_damage(materials)));
        }
        return tooltip;
    }

    pub(crate) fn make_texture(&self, resources: &Resources) -> Texture {
        if self.cached_placed_texture.borrow().is_none() {
            let mut map = HashMap::new();
            if let Some(material) = &self.material {
                map = material.pallete_sprite(&resources.materials);
            }
            let placed_sprite = &resources.item_blueprints.get(&self.blueprint_id).placed_sprite;
            let image = placed_sprite.remap(map);
            self.cached_placed_texture.borrow_mut().replace(image);
        }
        let image = self.cached_placed_texture.borrow();
        let image = image.as_ref().expect("Just populated");
        let settings = TextureSettings::new().filter(Filter::Nearest);
        return Texture::from_image(&image, &settings)
    }

    pub(crate) fn make_inventory_texture(&self, index: usize, resources: &Resources) -> Texture {
        if self.cached_inventory_texture.borrow().is_none() {
            let mut map = HashMap::new();
            if let Some(material) = &self.material {
                map = material.pallete_sprite(&resources.materials);
            }
            let inventory_sprite = &resources.item_blueprints.get(&self.blueprint_id).inventory_sprite;
            let image = inventory_sprite.remap(map);
            let image = DynamicImage::ImageRgba8(image);
            let image = image.crop_imm((index * SPECIES_SPRITE_SIZE.x()) as u32, 0, SPECIES_SPRITE_SIZE.x() as u32, SPECIES_SPRITE_SIZE.y() as u32).to_rgba8();
            self.cached_inventory_texture.borrow_mut().replace(image);
        }
        let image = self.cached_inventory_texture.borrow();
        let image = image.as_ref().expect("Just populated");
        let settings = TextureSettings::new().filter(Filter::Nearest);
        return Texture::from_image(&image, &settings)
    }

    pub(crate) fn total_damage(&self, materials: &Materials) -> DamageRoll {
        let damage = self.extra_damage(materials);
        if let Some(weapon_damage) = &self.mellee_damage {
            return damage + weapon_damage.damage.clone();
        }
        return damage
    }

    pub(crate) fn extra_damage(&self, materials: &Materials) -> DamageRoll {
        let mut damage = DamageRoll::empty();
        if let Some(material) = &self.material {
            let primary = materials.get(&material.primary);
            damage = damage + primary.extra_damage.clone();
            if let Some(secondary) = &material.secondary {
                let secondary = materials.get(secondary);
                damage = damage + secondary.extra_damage.clone();
            }
            if let Some(details) = &material.details {
                let details = materials.get(details);
                damage = damage + details.extra_damage.clone();
            }
        }
        return damage;
    }

}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ActionProviderComponent {
    pub(crate) actions: Vec<ActionId>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct QualityComponent {
    pub(crate) quality: ItemQuality,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MaterialComponent {
    pub(crate) primary: MaterialId,
    pub(crate) secondary: Option<MaterialId>,
    pub(crate) details: Option<MaterialId>,
}

impl MaterialComponent {

    fn pallete_sprite(&self, materials: &Materials) -> HashMap<ColorMap, [Color; 4]> {
        let mut map = HashMap::new();
        map.insert(ColorMap::Blue, materials.get(&self.primary).color_pallete);
        if let Some(secondary) = self.secondary {
            map.insert(ColorMap::Green, materials.get(&secondary).color_pallete);
        }
        if let Some(details) = self.details {
            map.insert(ColorMap::Red, materials.get(&details).color_pallete);
        }
        return map;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MelleeDamageComponent {
    pub(crate) damage: DamageRoll,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ArmorComponent {
    pub(crate) protection: DamageModel,
    pub(crate) coverage: Vec<BodyPart>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ArtworkSceneComponent {
    pub(crate) scene: ArtworkScene,
}

pub(crate) enum ItemMakeArguments {
    PrimaryMaterial(MaterialId),
    SecondaryMaterial(MaterialId),
    DetailsMaterial(MaterialId),
    Quality(ItemQuality),
    Scene(ArtworkScene),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum ArtworkScene {
    Bust { creature_id: CreatureId },
    FullBody { creature_id: CreatureId, artifact_id: Option<ItemId> }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) enum ItemQuality {
    Poor,
    Normal,
    Good,
    Excelent,
    Legendary
}

impl ItemQuality {
    pub(crate) fn main_stat_multiplier(&self) -> f32 {
        match self {
            Self::Poor => 0.8,
            Self::Normal => 1.,
            Self::Good => 1.2,
            Self::Excelent => 1.5,
            Self::Legendary => 2.,
        }
    }
}