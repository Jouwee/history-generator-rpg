use std::{cell::RefCell, collections::HashMap};

use image::{DynamicImage, RgbaImage};
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::{commons::damage_model::{DamageModel, DamageRoll}, engine::pallete_sprite::{ColorMap, PalleteSprite}, game::{actor::health_component::BodyPart, inventory::inventory::EquipmentType}, resources::{action::ActionId, material::{MaterialId, Materials}, species::SPECIES_SPRITE_SIZE}, Color, Resources};

use super::{creature::CreatureId, world::World};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct ItemId(usize);
impl crate::commons::id_vec::Id for ItemId {
    fn new(id: usize) -> Self {
        Self(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Item {
    pub(crate) name: String,
    pub(crate) special_name: Option<String>,
    pub(crate) owner: Option<CreatureId>,
    pub(crate) placed_sprite: PalleteSprite,
    pub(crate) action_provider: Option<ActionProviderComponent>,
    pub(crate) equippable: Option<EquippableComponent>,
    pub(crate) material: Option<MaterialComponent>,
    pub(crate) quality: Option<QualityComponent>,
    pub(crate) mellee_damage: Option<MelleeDamageComponent>,
    pub(crate) armor: Option<ArmorComponent>,
    pub(crate) artwork_scene: Option<ArtworkSceneComponent>
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

    pub(crate) fn description(&self, resources: &Resources, world: &World) -> String {
        let mut description = self.name.clone();

        if let Some(quality) = &self.quality {
            description = format!("{:?} {description}", quality.quality)
        }

        if let Some(material) = &self.material {
            let mut composition = Vec::new();
            
            let primary = resources.materials.get(&material.primary);
            composition.push(primary.name.clone());

            if let Some(secondary) = material.secondary {
                let primary = resources.materials.get(&secondary);
                composition.push(primary.name.clone());
            }

            if let Some(details) = material.details {
                let primary = resources.materials.get(&details);
                composition.push(primary.name.clone());
            }

            let composition = composition.join(", ");
            description = format!("{description} made of {composition}");
        }

        if let Some(scene) = &self.artwork_scene {
            match scene.scene {
                ArtworkScene::Bust { creature_id } => {
                    let creature = world.creatures.get(&creature_id);
                    description = format!("{description}. It depicts a bust of {}", creature.name(&creature_id, world, resources));
                },
                ArtworkScene::FullBody { creature_id, artifact_id } => {
                    let creature = world.creatures.get(&creature_id);
                    description = match artifact_id {
                        Some(artifact) => {
                            let artifact = world.artifacts.get(&artifact);
                            format!("{description}. It depicts a full-body image of {} holding {}", creature.name(&creature_id, world, resources), artifact.name(&resources.materials))
                        }
                        None => format!("{description}. It depicts a full-body image of {}", creature.name(&creature_id, world, resources))
                    };                    
                }
            }
        }
        
        return description
    }

    pub(crate) fn make_texture(&self, materials: &Materials) -> Texture {
        let mut map = HashMap::new();
        if let Some(material) = &self.material {
            map = material.pallete_sprite(materials);
        }
        let image = self.placed_sprite.remap(map);
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


#[derive(Clone, Debug)]
pub(crate) struct ActionProviderComponent {
    pub(crate) actions: Vec<ActionId>
}

#[derive(Clone, Debug)]
pub(crate) struct EquippableComponent {
    pub(crate) sprite: PalleteSprite,
    pub(crate) slot: EquipmentType,
    pub(crate) cached_texture: RefCell<Option<RgbaImage>>
}

impl EquippableComponent {

    pub(crate) fn make_texture(&self, index: usize, material: &Option<MaterialComponent>, materials: &Materials) -> Texture {
        if self.cached_texture.borrow().is_none() {
            let mut map = HashMap::new();
            if let Some(material) = &material {
                map = material.pallete_sprite(materials);
            }
            let image = self.sprite.remap(map);
            let image = DynamicImage::ImageRgba8(image);
            let image = image.crop_imm((index * SPECIES_SPRITE_SIZE.x()) as u32, 0, SPECIES_SPRITE_SIZE.x() as u32, SPECIES_SPRITE_SIZE.y() as u32).to_rgba8();
            self.cached_texture.borrow_mut().replace(image);
        }

        let image = self.cached_texture.borrow();
        let image = image.as_ref().expect("Just populated");
        let settings = TextureSettings::new().filter(Filter::Nearest);
        return Texture::from_image(&image, &settings)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct QualityComponent {
    pub(crate) quality: ItemQuality,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub(crate) struct MelleeDamageComponent {
    pub(crate) damage: DamageRoll,
}

#[derive(Clone, Debug)]
pub(crate) struct ArmorComponent {
    pub(crate) protection: DamageModel,
    pub(crate) coverage: Vec<BodyPart>
}


#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub(crate) enum ArtworkScene {
    Bust { creature_id: CreatureId },
    FullBody { creature_id: CreatureId, artifact_id: Option<ItemId> }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum ItemQuality {
    Poor,
    Normal,
    Good,
    Excelent,
    Legendary
}

impl ItemQuality {
    pub(crate) fn main_stat_modifier(&self) -> i16 {
        match self {
            Self::Poor => -1,
            Self::Normal => 0,
            Self::Good => 1,
            Self::Excelent => 2,
            Self::Legendary => 4,
        }
    }
}