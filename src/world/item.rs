use std::collections::HashMap;

use image::ImageReader;
use opengl_graphics::Texture;

use crate::{commons::rng::Rng, engine::pallete_sprite::{ColorMap, PalleteSprite}, game::action::ActionId, resources::resources::{Actions, Materials}};

use super::material::MaterialId;

#[derive(Clone, Debug)]
pub enum Item {
    Sword(Sword),
    Mace(Mace),
    Lance(Lance),
}

impl Item {

    pub fn name(&self, materials: &Materials) -> String {
        match self {
            Item::Sword(sword) => {
                if let Some(name) = &sword.name {
                    return name.clone()
                }
                let blade = materials.get(&sword.blade_mat).name.clone();
                return format!("{blade} sword")
            },
            Item::Mace(mace) => {
                if let Some(name) = &mace.name {
                    return name.clone()
                }
                let head = materials.get(&mace.head_mat).name.clone();
                return format!("{head} mace")
            },
            Item::Lance(lance) => {
                if let Some(name) = &lance.name {
                    return name.clone()
                }
                let tip = materials.get(&lance.tip_mat).name.clone();
                return format!("{tip} lance")
            }
        }
    }

    pub fn description(&self, materials: &Materials) -> String {
        let str;
        match self {
            Item::Sword(sword) => {
                let handle = materials.get(&sword.handle_mat).name.clone();
                let blade = materials.get(&sword.blade_mat).name.clone();
                let pommel = materials.get(&sword.pommel_mat).name.clone();
                let guard = materials.get(&sword.guard_mat).name.clone();
                str = format!("It's a sword. It's blade is made of {blade}. The handle, made of {handle} is topped by a pomel of {pommel} and a guard of {guard}.");
            },
            Item::Mace(mace) => {
                let handle = materials.get(&mace.handle_mat).name.clone();
                let head = materials.get(&mace.head_mat).name.clone();
                let pommel = materials.get(&mace.pommel_mat).name.clone();
                str = format!("It's a mace. It's head is made of {head}. The handle, made of {handle} is topped by a pomel of {pommel}.");
            },
            Item::Lance(lance) => {
                let handle = materials.get(&lance.handle_mat).name.clone();
                let tip = materials.get(&lance.tip_mat).name.clone();
                str = format!("It's a lance. It's tip is made of {tip}, with a handle made of {handle}.");
            }
        }
        return str
    }

    pub fn actions(&self, actions: &Actions) -> Vec<ActionId> {
        match self {
            Item::Sword(_sword) => {
                return vec!(actions.id_of("act:sword:slash"), actions.id_of("act:sword:bleeding_cut"))
            },
            Item::Mace(_mace) => {
                return vec!(actions.id_of("act:sword:slash"))
            },
            Item::Lance(_lance) => {
                return vec!(actions.id_of("act:sword:slash"))
            }
        }
    }


    pub fn make_texture(&self, materials: &Materials) -> Texture {
        match self {
            Self::Sword(sword) => {
                let image = ImageReader::open("./assets/sprites/sword.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, materials.get(&sword.blade_mat).color_pallete);
                map.insert(ColorMap::Red, materials.get(&sword.guard_mat).color_pallete);
                map.insert(ColorMap::Green, materials.get(&sword.handle_mat).color_pallete);
                map.insert(ColorMap::Yellow, materials.get(&sword.pommel_mat).color_pallete);
                return pallete_sprite.remap(map)
            },
            Self::Mace(mace) => {
                let image = ImageReader::open("./assets/sprites/mace.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, materials.get(&mace.head_mat).color_pallete);
                map.insert(ColorMap::Yellow, materials.get(&mace.handle_mat).color_pallete);
                map.insert(ColorMap::Green, materials.get(&mace.pommel_mat).color_pallete);
                return pallete_sprite.remap(map)
            },
            Self::Lance(lance) => {
                let image = ImageReader::open("./assets/sprites/lance.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, materials.get(&lance.tip_mat).color_pallete);
                map.insert(ColorMap::Yellow, materials.get(&lance.handle_mat).color_pallete);
                return pallete_sprite.remap(map)
            }
        }
    }

    pub fn make_equipped_texture(&self, materials: &Materials) -> Texture {
        match self {
            Self::Sword(sword) => {
                let image = ImageReader::open("./assets/sprites/species/human/sword_equipped.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, materials.get(&sword.blade_mat).color_pallete);
                map.insert(ColorMap::Red, materials.get(&sword.guard_mat).color_pallete);
                map.insert(ColorMap::Green, materials.get(&sword.handle_mat).color_pallete);
                map.insert(ColorMap::Yellow, materials.get(&sword.pommel_mat).color_pallete);
                return pallete_sprite.remap(map)
            },
            Self::Mace(mace) => {
                let image = ImageReader::open("./assets/sprites/species/human/mace_equipped.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, materials.get(&mace.head_mat).color_pallete);
                map.insert(ColorMap::Yellow, materials.get(&mace.handle_mat).color_pallete);
                map.insert(ColorMap::Green, materials.get(&mace.pommel_mat).color_pallete);
                return pallete_sprite.remap(map)
            },
            Self::Lance(lance) => {
                let image = ImageReader::open("./assets/sprites/species/human/lance_equipped.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, materials.get(&lance.tip_mat).color_pallete);
                map.insert(ColorMap::Yellow, materials.get(&lance.handle_mat).color_pallete);
                return pallete_sprite.remap(map)
            }
        }
    }

    pub fn damage_mult(&self) -> f32 { 
        match self {
            Item::Sword(sword) => sword.damage_mult,
            Item::Lance(sword) => sword.damage_mult,
            Item::Mace(sword) => sword.damage_mult,
        }
    }

}

#[derive(Clone, Debug)]
pub enum ItemQuality {
    Poor,
    Normal,
    Good,
    Excelent,
    Legendary
}

impl ItemQuality {
    pub fn main_stat_multiplier(&self) -> f32 {
        match self {
            Self::Poor => 0.7,
            Self::Normal => 1.0,
            Self::Good => 1.2,
            Self::Excelent => 1.4,
            Self::Legendary => 2.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Sword {
    pub quality: ItemQuality,
    pub handle_mat: MaterialId,
    pub blade_mat: MaterialId,
    pub pommel_mat: MaterialId,
    pub guard_mat: MaterialId,
    pub damage_mult: f32,
    pub name: Option<String>
}

impl Sword {
    pub fn new(quality: ItemQuality, handle_mat: MaterialId, blade_mat: MaterialId, pommel_mat: MaterialId, guard_mat: MaterialId, materials: &Materials) -> Sword {
        let blade = materials.get(&blade_mat);
        let damage_mult = blade.sharpness * quality.main_stat_multiplier();
        Sword { quality, handle_mat, blade_mat, pommel_mat, guard_mat, damage_mult, name: None }
    }
}

#[derive(Clone, Debug)]
pub struct Mace {
    pub quality: ItemQuality,
    pub handle_mat: MaterialId,
    pub head_mat: MaterialId,
    pub pommel_mat: MaterialId,
    pub damage_mult: f32,
    pub name: Option<String>
}

impl Mace {
    pub fn new(quality: ItemQuality, handle_mat: MaterialId, head_mat: MaterialId, pommel_mat: MaterialId, materials: &Materials) -> Mace {
        let head = materials.get(&head_mat);
        let damage_mult = head.sharpness * quality.main_stat_multiplier();
        Mace { quality, handle_mat, head_mat, pommel_mat, damage_mult, name: None }
    }
}

#[derive(Clone, Debug)]
pub struct Lance {
    pub quality: ItemQuality,
    pub handle_mat: MaterialId,
    pub tip_mat: MaterialId,
    pub damage_mult: f32,
    pub name: Option<String>
}

impl Lance {
    pub fn new(quality: ItemQuality, handle_mat: MaterialId, tip_mat: MaterialId, materials: &Materials) -> Lance {
        let tip = materials.get(&tip_mat);
        let damage_mult = tip.sharpness * quality.main_stat_multiplier();
        Lance { quality, handle_mat, tip_mat, damage_mult, name: None }
    }
}

pub struct ItemMaker {}

impl ItemMaker {

    pub fn random(rng: &Rng, materials: &Materials, quality: ItemQuality) -> Item {
        let mut rng = rng.derive("random_item");
        let item = rng.randu_range(0, 3);

        let blades = [materials.id_of("mat:steel"), materials.id_of("mat:bronze"), materials.id_of("mat:copper")];
        let blade = blades[rng.randu_range(0, blades.len())];
        let handles = [materials.id_of("mat:oak"), materials.id_of("mat:birch")];
        let handle = handles[rng.randu_range(0, handles.len())];
        let extras = [materials.id_of("mat:steel"), materials.id_of("mat:bronze"), materials.id_of("mat:copper")];
        let extra = extras[rng.randu_range(0, extras.len())];

        match item {
            0 => Item::Sword(Sword::new(quality, handle, blade, extra, extra, materials)),
            1 => Item::Mace(Mace::new(quality, handle, blade, extra, materials)),
            _ => Item::Lance(Lance::new(quality, handle, blade, materials))
        }
    }

}