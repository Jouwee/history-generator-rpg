use crate::{commons::resource_map::ResourceMap, engine::Color};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash, Eq)]
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

#[derive(Clone, Debug)]
pub(crate) struct Material {
    pub(crate) name: String,
    pub(crate) sharpness: f32,
    pub(crate) color_pallete: [Color; 4]
}

impl Material {

    pub(crate) fn new_metal(name: &str) -> Material {
        Material {
            name: name.to_string(),
            sharpness: 1.,
            color_pallete: [Color::from_hex("405273"), Color::from_hex("6c81a1"), Color::from_hex("96a9c1"), Color::from_hex("bbc3d0")]
        }
    }

    pub(crate) fn new_wood(name: &str) -> Material {
        Material {
            name: name.to_string(),
            sharpness: 0.1,
            color_pallete: [Color::from_hex("3d3333"), Color::from_hex("593e47"), Color::from_hex("7a5859"), Color::from_hex("a57855")]
        }
    }

}