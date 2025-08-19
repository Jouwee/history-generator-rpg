use std::collections::HashMap;

use opengl_graphics::Texture;
use serde::{Deserialize, Serialize};

use crate::{commons::id_vec::Id, engine::{audio::SoundEffect, geometry::{Coord2, Size2D}, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, tilemap::{TileMap, TileSet}}, resources::{object_tile::ObjectTileId, resources::Resources, tile::TileId}, world::{creature::CreatureId, item::Item}};

pub(crate) struct Chunk {
    pub(crate) coord: ChunkCoord,
    pub(crate) size: Size2D,
    pub(crate) tiles_metadata: HashMap<Coord2, TileMetadata>,
    pub(crate) items_on_ground: Vec<(Coord2, Item, Texture)>,
    pub(crate) ground_layer: LayeredDualgridTilemap,
    pub(crate) object_layer: TileMap,
}

/// This is used as a temporary value during deserialization, but the chunk in this state is not useful
impl Default for Chunk {

    fn default() -> Self {
        Self {
            coord: ChunkCoord::new(Coord2::xy(1, 1), ChunkLayer::Surface),
            size: Size2D(1, 1),
            tiles_metadata: HashMap::new(),
            items_on_ground: Vec::new(),
            ground_layer: LayeredDualgridTilemap::new(LayeredDualgridTileset::new(), 1, 1, 1, 1),
            object_layer: TileMap::new(TileSet::new(), 1, 1, 1, 1)
        }
    }

}

impl Chunk {

    pub(crate) fn new(coord: ChunkCoord, size: Size2D, resources: &Resources) -> Self {
        let mut tileset = TileSet::new();
        for tile in resources.object_tiles.iter() {
            tileset.add(tile.tile.clone());    
        }

        let mut dual_tileset = LayeredDualgridTileset::new();
        for tile in resources.tiles.iter() {
            dual_tileset.add(tile.tile_layer, tile.tileset_image.clone());
        }
        
        Chunk {
            coord,
            size,
            ground_layer: LayeredDualgridTilemap::new(dual_tileset, size.x(), size.y(), 24, 24),
            object_layer: TileMap::new(tileset, size.x(), size.y(), 24, 24),
            items_on_ground: Vec::new(),
            tiles_metadata: HashMap::new(),
        }
    }

    pub(crate) fn blocks_movement(&self, pos: Coord2) -> bool {
        if let crate::engine::tilemap::Tile::Empty = self.object_layer.get_tile(pos.x as usize, pos.y as usize) {
            return false
        }
        // TODO: Resources
        let i = self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize);
        if i == 9 || i == 11 || i == 12 || i == 16 || i == 17 {
            return false
        }
        return true
    }

    pub(crate) fn check_line_of_sight(&self, from: &Coord2, to: &Coord2) -> bool {
        let angle_degrees = f64::atan2((to.y - from.y) as f64, (to.x - from.x) as f64);
        let dist = from.dist(to) as f64;
        let mut step = 0.;

        let mut pos = from.clone();
        let mut last = pos.clone();
        while step < dist {
            if pos != last {               
                if self.blocks_movement(pos) {
                    return false;
                }
                last = pos.clone();
            }
            step += 0.1;
            pos = Coord2::xy(
                from.x + (step * angle_degrees.cos()) as i32,
                from.y + (step * angle_degrees.sin()) as i32,
             );
         }
        return true;
    }

    // SMELL: This -1 +1 thing is prone to errors
    pub(crate) fn set_object_key(&mut self, pos: Coord2, tile: &str, resources: &Resources) {
        let id = resources.object_tiles.id_of(tile);
        let shadow = resources.object_tiles.get(&id).casts_shadow;
        self.object_layer.set_tile(pos.x as usize, pos.y as usize, id.as_usize() + 1);
        self.object_layer.set_shadow(pos.x as usize, pos.y as usize, shadow);
    }

    pub(crate) fn set_object_idx(&mut self, pos: Coord2, id: usize, resources: &Resources) {
        // SMELL
        let shadow;
        if id > 0 {
            shadow = resources.object_tiles.try_get(id - 1).unwrap().casts_shadow;
        } else {
            shadow = false;
        }
        self.object_layer.set_tile(pos.x as usize, pos.y as usize, id);
        self.object_layer.set_shadow(pos.x as usize, pos.y as usize, shadow);
    }

    pub(crate) fn get_object_idx(&self, pos: Coord2) -> usize {
        return self.object_layer.get_tile_idx(pos.x as usize, pos.y as usize)
    }
    
    pub(crate) fn remove_object(&mut self, pos: Coord2) {
        self.object_layer.set_tile(pos.x as usize, pos.y as usize, 0);
    }

    pub(crate) fn get_step_sound(&self, pos: Coord2, resources: &Resources) -> Option<SoundEffect> {
        if let Some(tile) = self.ground_layer.tile(pos.x as usize, pos.y as usize) {
            let tile = resources.tiles.try_get(tile);
            if let Some(tile) = tile {
                return tile.step_sound_effect.clone()
            }
        }
        None
    }

}

/// Serializable version of a chunk
#[derive(Serialize, Deserialize)]
pub(crate) struct ChunkSerialized {
    pub(crate) coord: ChunkCoord,
    pub(crate) size: Size2D,
    pub(crate) tiles_metadata: HashMap<Coord2, TileMetadata>,
    pub(crate) items_on_ground: Vec<(Coord2, Item)>,
    pub(crate) ground_layer: Vec<Option<TileId>>,
    pub(crate) object_layer: Vec<Option<ObjectTileId>>,
}

impl ChunkSerialized {
    pub(crate) fn from_chunk(chunk: &Chunk) -> Self {
        Self {
            coord: chunk.coord,
            size: chunk.size,
            tiles_metadata: chunk.tiles_metadata.clone(),
            items_on_ground: chunk.items_on_ground.iter().map(|i| (i.0, i.1.clone())).collect(),
            ground_layer: chunk.ground_layer.tiles().iter().map(|i| i.and_then(|i| Some(TileId::new(i)))).collect(),
            object_layer: chunk.object_layer.tiles().iter().map(|(i, _)| match i {
                0 => None,
                // SMELL: See other smells in this file
                i => Some(ObjectTileId::new(i - 1))
            }).collect()
        }
    }

    pub(crate) fn to_chunk(&self, resources: &Resources) -> Chunk {
        let mut chunk = Chunk::new(self.coord, self.size, resources);

        chunk.tiles_metadata = self.tiles_metadata.clone();
        chunk.items_on_ground = self.items_on_ground.iter().map(|i| (i.0, i.1.clone(), i.1.make_texture(resources))).collect();

        for x in 0..self.size.x() {
            for y in 0..self.size.y() {
                let i = (y * self.size.x()) + x;
                if let Some(tile) = self.ground_layer[i] {
                    chunk.ground_layer.set_tile(x, y, tile.as_usize());
                }
                if let Some(tile) = self.object_layer[i] {
                    // SMELL: See other smells in this file
                    chunk.set_object_idx(Coord2::xy(x as i32, y as i32), tile.as_usize() + 1, resources);
                }
            }
        }
        return chunk;
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct ChunkCoord {
    pub(crate) xy: Coord2,
    pub(crate) layer: ChunkLayer,    
}

impl ChunkCoord {

    pub(crate) fn new(xy: Coord2, layer: ChunkLayer) -> Self {
        return ChunkCoord { xy, layer }
    }

}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) enum ChunkLayer {
    Surface,
    Underground
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) enum TileMetadata {
    BurialPlace(CreatureId)
}