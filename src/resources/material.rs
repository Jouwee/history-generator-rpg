use serde::{Deserialize, Serialize};

use crate::{commons::{damage_model::{DamageRoll}, resource_map::ResourceMap}, engine::Color};

// TODO(0xtBbih5): Should serialize the string id, not the internal id
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq, Serialize, Deserialize)]
pub(crate) struct MaterialId(usize);
impl crate::commons::id_vec::Id for MaterialId {
    fn new(id: usize) -> Self {
        MaterialId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Materials = ResourceMap<MaterialId, Material>;

pub(crate) const MAT_TAG_METAL: u8 = 0b0000_0001;
pub(crate) const MAT_TAG_WOOD: u8 = 0b0000_0010;
pub(crate) const MAT_TAG_BONE: u8 = 0b0000_0100;
pub(crate) const MAT_TAG_LEATHER: u8 = 0b0000_1000;
pub(crate) const MAT_TAG_CLOTH: u8 = 0b0001_0000;

#[derive(Clone, Debug)]
pub(crate) struct Material {
    pub(crate) name: String,
    pub(crate) sharpness: f32,
    pub(crate) color_pallete: [Color; 4],
    pub(crate) tags_bitmask: u8,
    pub(crate) extra_damage: DamageRoll,
}

impl Material {

    pub(crate) fn new_metal(name: &str) -> Material {
        Material {
            name: name.to_string(),
            sharpness: 1.,
            color_pallete: [Color::from_hex("405273"), Color::from_hex("6c81a1"), Color::from_hex("96a9c1"), Color::from_hex("bbc3d0")],
            tags_bitmask: MAT_TAG_METAL,
            extra_damage: DamageRoll::empty(),
        }
    }

    pub(crate) fn new_wood(name: &str) -> Material {
        Material {
            name: name.to_string(),
            sharpness: 0.5,
            color_pallete: [Color::from_hex("3d3333"), Color::from_hex("593e47"), Color::from_hex("7a5859"), Color::from_hex("a57855")],
            tags_bitmask: MAT_TAG_WOOD,
            extra_damage: DamageRoll::empty(),
        }
    }

    pub(crate) fn new_bone(name: &str) -> Material {
        Material {
            name: name.to_string(),
            sharpness: 0.8,
            color_pallete: [Color::from_hex("d4c692"), Color::from_hex("fee1b8"), Color::from_hex("f1f6f0"), Color::from_hex("f1f6f0")],
            tags_bitmask: MAT_TAG_BONE,
            extra_damage: DamageRoll::empty(),
        }
    }

    pub(crate) fn new_leather(name: &str) -> Material {
        Material {
            name: name.to_string(),
            sharpness: 0.8,
            color_pallete: [Color::from_hex("413028"), Color::from_hex("604332"), Color::from_hex("8d634a"), Color::from_hex("604332")],
            tags_bitmask: MAT_TAG_BONE,
            extra_damage: DamageRoll::empty(),
        }
    }

    pub(crate) fn new_cloth(name: &str) -> Material {
        Material {
            name: name.to_string(),
            sharpness: 0.8,
            color_pallete: [Color::from_hex("524235"), Color::from_hex("6e6750"), Color::from_hex("92825c"), Color::from_hex("92825c")],
            tags_bitmask: MAT_TAG_BONE,
            extra_damage: DamageRoll::empty(),
        }
    }

}