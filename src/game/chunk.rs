use crate::{commons::{history_vec::Id, rng::Rng}, engine::Point2D, WorldGraph};
use super::NPC;

pub struct Chunk {
    size: ChunkSize,
    tiles: Vec<Tile>,
    pub npcs: Vec<NPC>,
    pub killed_people: Vec<Id>
}

impl Chunk {
    pub fn new(size: ChunkSize) -> Chunk {
        Chunk {
            size,
            tiles: Vec::with_capacity(size.area()),
            npcs: Vec::new(),
            killed_people: Vec::new()
        }
    }

    pub fn from_world_tile(world: &WorldGraph, xy: Point2D) -> Chunk {
        let mut chunk = Self::new(ChunkSize(64, 64));
        let mut rng = Rng::rand();
        for (id, person) in world.people.iter() {
            if person.borrow().position == xy {
                let point = Point2D(
                    rng.randu_range(0, chunk.size.x()),
                    rng.randu_range(0, chunk.size.y())
                );
                chunk.npcs.push(NPC::new(point, *id, &person.borrow()));
            }
        }
        chunk
    }
}

#[derive(Clone, Copy)]
pub struct ChunkSize(usize, usize);

impl ChunkSize {
    pub fn x(&self) -> usize {
        self.0
    }
    pub fn y(&self) -> usize {
        self.1
    }
    pub fn area(&self) -> usize {
        return self.x() * self.y()
    }
}

pub struct Tile {

}