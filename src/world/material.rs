use crate::engine::Color;

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub material_type: MaterialType,
    pub density: f32,
    pub sharpness: f32,
    pub color_pallete: [Color; 4]
}

impl Material {

    pub fn new_metal(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Metal,
            density: 7.85,
            sharpness: 1.,
            color_pallete: [Color::from_hex("405273"), Color::from_hex("6c81a1"), Color::from_hex("96a9c1"), Color::from_hex("bbc3d0")]
        }
    }

    pub fn new_wood(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Wood,
            density: 0.5,
            sharpness: 0.1,
            color_pallete: [Color::from_hex("3d3333"), Color::from_hex("593e47"), Color::from_hex("7a5859"), Color::from_hex("a57855")]
        }
    }

    pub fn new_bone(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Bone,
            density: 0.5,
            sharpness: 0.3,
            color_pallete: [Color::from_hex("d4c692"), Color::from_hex("fee1b8"), Color::from_hex("f1f6f0"), Color::from_hex("f1f6f0")]
        }
    }

}

#[derive(Debug)]
pub enum MaterialType {
    Metal,
    Wood,
    Bone,
}