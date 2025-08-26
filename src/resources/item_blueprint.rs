use std::cell::RefCell;

use serde::{Deserialize, Serialize};

use crate::{commons::{damage_model::{DamageModel, DamageRoll}, id_vec::Identified, resource_map::ResourceMap}, engine::pallete_sprite::PalleteSprite, game::{actor::health_component::BodyPart, inventory::inventory::EquipmentType}, resources::action::Affliction, world::item::{ActionProviderComponent, ArmorComponent, ArtworkSceneComponent, ItemMakeArguments, MaterialComponent, MelleeDamageComponent, QualityComponent}, Item, Resources};

// TODO(ROO4JcDl): Should serialize the string id, not the internal id
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct ItemBlueprintId(usize);
impl crate::commons::id_vec::Id for ItemBlueprintId {
    fn new(id: usize) -> Self {
        Self(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type ItemBlueprints = ResourceMap<ItemBlueprintId, ItemBlueprint>;

#[derive(Clone)]
pub(crate) struct ItemBlueprint {
    pub(crate) name: String,
    pub(crate) placed_sprite: PalleteSprite,
    pub(crate) inventory_sprite: PalleteSprite,
    pub(crate) action_provider: Option<ActionProviderComponent>,
    pub(crate) equippable: Option<EquippableComponent>,
    pub(crate) material: Option<MaterialBlueprintComponent>,
    pub(crate) quality: Option<QualityBlueprintComponent>,
    pub(crate) mellee_damage: Option<MelleeDamageBlueprintComponent>,
    pub(crate) armor: Option<ArmorBlueprintComponent>,
    pub(crate) artwork_scene: Option<ArtworkSceneBlueprintComponent>,
    pub(crate) name_blueprint: Option<NameBlueprintComponent>,
    pub(crate) consumable: Option<ConsumableComponent>,
}

pub(crate) trait ItemMaker {
    fn make(&self, arguments: Vec<ItemMakeArguments>, resources: &Resources) -> Item;
}

impl<'_a> ItemMaker for Identified<'_a, ItemBlueprintId, ItemBlueprint> {

    fn make(&self, arguments: Vec<ItemMakeArguments>, resources: &Resources) -> Item {
        Item {
            blueprint_id: *self.id(),
            name: self.name.clone(),
            special_name: None,
            action_provider: self.action_provider.clone(),
            // equippable: self.equippable.clone(),
            owner: None,
            material: self.material.as_ref().map(|material_blueprint| material_blueprint.make(&arguments)),
            quality: self.quality.as_ref().map(|quality_blueprint| quality_blueprint.make(&arguments)),
            mellee_damage: self.mellee_damage.as_ref().map(|mellee_blueprint| mellee_blueprint.make(&arguments, &resources)),
            armor: self.armor.as_ref().map(|armor| armor.make(&arguments, &resources)),
            artwork_scene: self.artwork_scene.as_ref().map(|artwork_scene| artwork_scene.make(&arguments)),
            cached_inventory_texture: RefCell::new(None),
            cached_placed_texture: RefCell::new(None)
        }
    }

}


#[derive(Clone, Debug)]
pub(crate) struct MaterialBlueprintComponent {
    pub(crate) primary_tag_bitmask: u8,
    pub(crate) secondary_tag_bitmask: Option<u8>,
    pub(crate) details_tag_bitmask: Option<u8>,
}

impl MaterialBlueprintComponent {
    
    fn make(&self, arguments: &Vec<ItemMakeArguments>) -> MaterialComponent {
        let mut primary = None;
        let mut secondary = None;
        let mut details = None;
        
        for argument in arguments.iter() {
            match argument {
                ItemMakeArguments::PrimaryMaterial(material) => primary = Some(*material),
                ItemMakeArguments::SecondaryMaterial(material) => secondary = Some(*material),
                ItemMakeArguments::DetailsMaterial(material) => details = Some(*material),
                _ => ()
            }
        }
        
        if let Some(primary) = primary {
            return MaterialComponent { primary, secondary, details }
        } else {
            panic!("No primary material specified")
        }
    }

}

#[derive(Clone, Debug)]
pub(crate) struct QualityBlueprintComponent {
}

impl QualityBlueprintComponent {

    fn make(&self, arguments: &Vec<ItemMakeArguments>) -> QualityComponent {
        let mut selected_quality = None;
        
        for argument in arguments.iter() {
            match argument {
                ItemMakeArguments::Quality(quality) => selected_quality = Some(*quality),
                _ => ()
            }
        }
        
        if let Some(quality) = selected_quality {
            return QualityComponent { quality }
        } else {
            panic!("No quality specified")
        }
    }

}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct EquippableComponent {
    pub(crate) slot: EquipmentType,
}


#[derive(Clone, Debug)]
pub(crate) struct MelleeDamageBlueprintComponent {
    pub(crate) base_damage: DamageRoll,
}

impl MelleeDamageBlueprintComponent {

    fn make(&self, arguments: &Vec<ItemMakeArguments>, resources: &Resources) -> MelleeDamageComponent {
        let mut damage = self.base_damage.clone();
        for argument in arguments.iter() {
            match argument {
                ItemMakeArguments::PrimaryMaterial(material) => {
                    let material = resources.materials.get(material);
                    damage = damage.multiply(material.sharpness);
                },
                ItemMakeArguments::Quality(quality) => {
                    damage = damage.multiply(quality.main_stat_multiplier());
                },
                _ => ()
            }
        }

        return MelleeDamageComponent {
            damage
        }
    }

}


#[derive(Clone, Debug)]
pub(crate) struct ArmorBlueprintComponent {
    pub(crate) protection: DamageModel,
    pub(crate) coverage: Vec<BodyPart>
}

impl ArmorBlueprintComponent {

    fn make(&self, arguments: &Vec<ItemMakeArguments>, resources: &Resources) -> ArmorComponent {
        let mut protection = self.protection.clone();
        for argument in arguments.iter() {
            match argument {
                ItemMakeArguments::PrimaryMaterial(material) => {
                    let material = resources.materials.get(material);
                    protection = protection.multiply(material.sharpness);
                },
                _ => ()
            }
        }

        return ArmorComponent {
            protection,
            coverage: self.coverage.clone(),
        }
    }

}

#[derive(Clone, Debug)]
pub(crate) struct NameBlueprintComponent {
    pub(crate) suffixes: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ArtworkSceneBlueprintComponent {
}

impl ArtworkSceneBlueprintComponent {

    fn make(&self, arguments: &Vec<ItemMakeArguments>) -> ArtworkSceneComponent {
        let mut selected_scene = None;
        
        for argument in arguments.iter() {
            match argument {
                ItemMakeArguments::Scene(scene) => selected_scene = Some(scene.clone()),
                _ => ()
            }
        }
        
        if let Some(scene) = selected_scene {
            return ArtworkSceneComponent { scene }
        } else {
            panic!("No artwork specified")
        }
    }

}

#[derive(Clone, Debug)]
pub(crate) struct ConsumableComponent {
    pub(crate) effects: Vec<Affliction>,
}