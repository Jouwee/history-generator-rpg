use crate::{commons::resource_map::ResourceMap, engine::audio::SoundEffect};

use super::tile::{Tile, TileId};

pub struct Resources {
    pub tiles: ResourceMap<TileId, Tile>
}

impl Resources {

    pub fn new() -> Resources {
        Resources { tiles: ResourceMap::new() }
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
    }

    pub fn tile(&self, id: &TileId) -> &Tile {
        return self.tiles.get(id);
    }

    pub fn find_tile(&self, key: &str) -> &Tile {
        return self.tiles.find(key);
    }

}