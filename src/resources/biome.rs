use crate::commons::resource_map::{IdentifiedResource, ResourceMap};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
pub(crate) struct BiomeId(usize);
impl crate::commons::id_vec::Id for BiomeId {
    fn new(id: usize) -> Self {
        BiomeId(id)
    }
    fn as_usize(&self) -> usize {
        self.0
    }
}

pub(crate) type Biomes = ResourceMap<BiomeId, Biome>;

impl Biomes {

    pub (crate) fn get_u8(&'_ self, id: u8) -> IdentifiedResource<'_, BiomeId, Biome> {
        return self.get(&BiomeId(id as usize));
    }

}

#[derive(Debug, Clone)]
pub(crate) struct Biome {
    pub(crate) elevation: (i32, i32),
    pub(crate) temperature: (u8, u8),
    pub(crate) vegetation: (f32, f32),
    pub(crate) soil_fertility_range: (f32, f32),
}