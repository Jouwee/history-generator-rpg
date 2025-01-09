use std::collections::HashMap;

use image::ImageReader;
use opengl_graphics::Texture;

use crate::{commons::{damage_model::DamageComponent, history_vec::Id, rng::Rng}, engine::pallete_sprite::{ColorMap, PalleteSprite}};

use super::world::World;

#[derive(Clone, Debug)]
pub enum Item {
    Sword(Sword),
    Mace(Mace),
    Lance(Lance),
}

impl Item {

    pub fn name(&self, world: &World) -> String {
        match self {
            Item::Sword(sword) => {
                if let Some(name) = &sword.name {
                    return name.clone()
                }
                let blade = world.materials.get(&sword.blade_mat).unwrap().name.clone();
                return format!("{blade} sword")
            },
            Item::Mace(mace) => {
                if let Some(name) = &mace.name {
                    return name.clone()
                }
                let head = world.materials.get(&mace.head_mat).unwrap().name.clone();
                return format!("{head} mace")
            },
            Item::Lance(lance) => {
                if let Some(name) = &lance.name {
                    return name.clone()
                }
                let tip = world.materials.get(&lance.tip_mat).unwrap().name.clone();
                return format!("{tip} lance")
            }
        }
    }

    pub fn description(&self, world: &World) -> String {
        let str;
        match self {
            Item::Sword(sword) => {
                let handle = world.materials.get(&sword.handle_mat).unwrap().name.clone();
                let blade = world.materials.get(&sword.blade_mat).unwrap().name.clone();
                let pommel = world.materials.get(&sword.pommel_mat).unwrap().name.clone();
                let guard = world.materials.get(&sword.guard_mat).unwrap().name.clone();
                str = format!("It's a sword. It's blade is made of {blade}. The handle, made of {handle} is topped by a pomel of {pommel} and a guard of {guard}.");
            },
            Item::Mace(mace) => {
                let handle = world.materials.get(&mace.handle_mat).unwrap().name.clone();
                let head = world.materials.get(&mace.head_mat).unwrap().name.clone();
                let pommel = world.materials.get(&mace.pommel_mat).unwrap().name.clone();
                str = format!("It's a mace. It's head is made of {head}. The handle, made of {handle} is topped by a pomel of {pommel}.");
            },
            Item::Lance(lance) => {
                let handle = world.materials.get(&lance.handle_mat).unwrap().name.clone();
                let tip = world.materials.get(&lance.tip_mat).unwrap().name.clone();
                str = format!("It's a lance. It's tip is made of {tip}, with a handle made of {handle}.");
            }
        }
        return str
    }

    pub fn damage_model(&self) -> DamageComponent {
        match self {
            Item::Sword(sword) => sword.damage,
            Item::Mace(mace) => mace.damage,
            Item::Lance(lance) => lance.damage
        }
    }


    pub fn make_texture(&self, world: &World) -> Texture {
        match self {
            Self::Sword(sword) => {
                let image = ImageReader::open("./assets/sprites/sword.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, world.materials.get(&sword.blade_mat).unwrap().color_pallete);
                map.insert(ColorMap::Red, world.materials.get(&sword.guard_mat).unwrap().color_pallete);
                map.insert(ColorMap::Green, world.materials.get(&sword.handle_mat).unwrap().color_pallete);
                map.insert(ColorMap::Yellow, world.materials.get(&sword.pommel_mat).unwrap().color_pallete);
                return pallete_sprite.remap(map)
            },
            Self::Mace(mace) => {
                let image = ImageReader::open("./assets/sprites/mace.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, world.materials.get(&mace.head_mat).unwrap().color_pallete);
                map.insert(ColorMap::Yellow, world.materials.get(&mace.handle_mat).unwrap().color_pallete);
                map.insert(ColorMap::Green, world.materials.get(&mace.pommel_mat).unwrap().color_pallete);
                return pallete_sprite.remap(map)
            },
            Self::Lance(lance) => {
                let image = ImageReader::open("./assets/sprites/lance.png").unwrap().decode().unwrap();
                let pallete_sprite = PalleteSprite::new(image);
                let mut map = HashMap::new();
                map.insert(ColorMap::Blue, world.materials.get(&lance.tip_mat).unwrap().color_pallete);
                map.insert(ColorMap::Yellow, world.materials.get(&lance.handle_mat).unwrap().color_pallete);
                return pallete_sprite.remap(map)
            }
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
    pub handle_mat: Id,
    pub blade_mat: Id,
    pub pommel_mat: Id,
    pub guard_mat: Id,
    pub damage: DamageComponent,
    pub name: Option<String>
}

impl Sword {
    pub fn new(quality: ItemQuality, handle_mat: Id, blade_mat: Id, pommel_mat: Id, guard_mat: Id, world: &World) -> Sword {
        let blade = world.materials.get(&blade_mat).unwrap();
        let damage = DamageComponent::new(blade.sharpness * quality.main_stat_multiplier(), 0., 0.);
        Sword { quality, handle_mat, blade_mat, pommel_mat, guard_mat, damage, name: None }
    }
}

#[derive(Clone, Debug)]
pub struct Mace {
    pub quality: ItemQuality,
    pub handle_mat: Id,
    pub head_mat: Id,
    pub pommel_mat: Id,
    pub damage: DamageComponent,
    pub name: Option<String>
}

impl Mace {
    pub fn new(quality: ItemQuality, handle_mat: Id, head_mat: Id, pommel_mat: Id, world: &World) -> Mace {
        let head = world.materials.get(&head_mat).unwrap();
        let damage = DamageComponent::new(0., 0., head.sharpness * quality.main_stat_multiplier());
        Mace { quality, handle_mat, head_mat, pommel_mat, damage, name: None }
    }
}

#[derive(Clone, Debug)]
pub struct Lance {
    pub quality: ItemQuality,
    pub handle_mat: Id,
    pub tip_mat: Id,
    pub damage: DamageComponent,
    pub name: Option<String>
}

impl Lance {
    pub fn new(quality: ItemQuality, handle_mat: Id, tip_mat: Id, world: &World) -> Lance {
        let tip = world.materials.get(&tip_mat).unwrap();
        let damage = DamageComponent::new(0., tip.sharpness * quality.main_stat_multiplier(), 0.);
        Lance { quality, handle_mat, tip_mat, damage, name: None }
    }
}

pub struct ItemMaker {}

impl ItemMaker {

    pub fn random(rng: &Rng, world: &World, quality: ItemQuality) -> Item {
        let mut rng = rng.derive("random_item");
        let item = rng.randu_range(0, 3);

        let blades = [Id(0), Id(1), Id(6)];
        let blade = blades[rng.randu_range(0, blades.len())];
        let handles = [Id(2), Id(3)];
        let handle = handles[rng.randu_range(0, handles.len())];
        let extras = [Id(0), Id(1), Id(6)];
        let extra = extras[rng.randu_range(0, extras.len())];

        match item {
            0 => Item::Sword(Sword::new(quality, handle, blade, extra, extra, world)),
            1 => Item::Mace(Mace::new(quality, handle, blade, extra, world)),
            _ => Item::Lance(Lance::new(quality, handle, blade, world))
        }
    }

}