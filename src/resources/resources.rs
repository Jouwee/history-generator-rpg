use crate::{commons::resource_map::ResourceMap, engine::{audio::SoundEffect, Color}, world::material::{Material, MaterialId}};

use super::tile::{Tile, TileId};

pub type Materials = ResourceMap<MaterialId, Material>;

#[derive(Clone)]
pub struct Resources {
    pub materials: Materials,
    pub tiles: ResourceMap<TileId, Tile>
}

impl Resources {

    pub fn new() -> Resources {
        Resources {
            tiles: ResourceMap::new(),
            materials: ResourceMap::new(),
        }
    }

    pub fn load(&mut self) {
        let mut tile = Tile::new(0, "assets/sprites/chunk_tiles/stone.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_stone_1.mp3", "sfx/step_stone_2.mp3", "sfx/step_stone_3.mp3")));
        self.tiles.add("tile:stone", tile);
        let mut tile = Tile::new(4, "assets/sprites/chunk_tiles/grass.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_grass_1.mp3", "sfx/step_grass_2.mp3", "sfx/step_grass_3.mp3")));
        self.tiles.add("tile:grass", tile);
        let tile = Tile::new(1, "assets/sprites/chunk_tiles/sand.png");
        self.tiles.add("tile:sand", tile);
        let tile = Tile::new(2, "assets/sprites/chunk_tiles/water.png");
        self.tiles.add("tile:water", tile);
        let mut tile = Tile::new(3, "assets/sprites/chunk_tiles/floor.png");
        tile.step_sound_effect = Some(SoundEffect::new(vec!("sfx/step_wood_1.mp3", "sfx/step_wood_2.mp3", "sfx/step_wood_3.mp3")));
        self.tiles.add("tile:floor", tile);
        
        self.materials.add("mat:steel", Material::new_metal("steel"));
        let mut bronze = Material::new_metal("bronze");
        bronze.color_pallete = [Color::from_hex("a57855"), Color::from_hex("de9f47"), Color::from_hex("fdd179"), Color::from_hex("fee1b8")];
        self.materials.add("mat:bronze", bronze);
        self.materials.add("mat:birch", Material::new_wood("birch"));
        self.materials.add("mat:oak", Material::new_wood("oak"));
        self.materials.add("mat:bone_leshen", Material::new_bone("leshen bone"));
        self.materials.add("mat:bone_fiend", Material::new_bone("fiend bone"));
        let mut copper = Material::new_metal("copper");
        copper.color_pallete = [Color::from_hex("593e47"), Color::from_hex("b55945"), Color::from_hex("de9f47"), Color::from_hex("f2b888")];
        self.materials.add("mat:copper", copper);

    }

    pub fn tile(&self, id: &TileId) -> &Tile {
        return self.tiles.get(id);
    }

    pub fn find_tile(&self, key: &str) -> &Tile {
        return self.tiles.find(key);
    }

}