use crate::{commons::{damage_model::DamageComponent, resource_map::ResourceMap}, engine::pallete_sprite::PalleteSprite, world::item::{ActionProviderComponent, ArmorComponent, ArtworkSceneComponent, EquippableComponent, ItemMakeArguments, MaterialComponent, MelleeDamageComponent, QualityComponent}, Item, Resources};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
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
    pub(crate) action_provider: Option<ActionProviderComponent>,
    pub(crate) equippable: Option<EquippableComponent>,
    pub(crate) material: Option<MaterialBlueprintComponent>,
    pub(crate) quality: Option<QualityBlueprintComponent>,
    pub(crate) mellee_damage: Option<MelleeDamageBlueprintComponent>,
    pub(crate) armor: Option<ArmorComponent>,
    pub(crate) artwork_scene: Option<ArtworkSceneBlueprintComponent>,
    pub(crate) name_blueprint: Option<NameBlueprintComponent>,
}

impl ItemBlueprint {

    pub(crate) fn make(&self, arguments: Vec<ItemMakeArguments>, resources: &Resources) -> Item {
        Item {
            name: self.name.clone(),
            special_name: None,
            placed_sprite: self.placed_sprite.clone(),
            action_provider: self.action_provider.clone(),
            equippable: self.equippable.clone(),
            material: match &self.material {
                Some(material_blueprint) => Some(material_blueprint.make(&arguments)),
                None => None,
            },
            quality: match &self.quality {
                Some(quality_blueprint) => Some(quality_blueprint.make(&arguments)),
                None => None,
            },
            mellee_damage: match &self.mellee_damage {
                Some(mellee_blueprint) => Some(mellee_blueprint.make(&arguments, &resources)),
                None => None
            },
            armor: self.armor.clone(),
            artwork_scene: match &self.artwork_scene {
                Some(artwork_scene) => Some(artwork_scene.make(&arguments)),
                None => None
            },
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
            panic!("No primary material specified")
        }
    }

}


#[derive(Clone, Debug)]
pub(crate) struct MelleeDamageBlueprintComponent {
    pub(crate) base_damage: DamageComponent,
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
            panic!("No primary material specified")
        }
    }

}