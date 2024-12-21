#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub material_type: MaterialType,
    pub density: f32,
    pub sharpness: f32,
}

impl Material {

    pub fn new_metal(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Metal,
            density: 7.85,
            sharpness: 1.
        }
    }

    pub fn new_wood(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Wood,
            density: 0.5,
            sharpness: 0.1
        }
    }

    pub fn new_bone(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Bone,
            density: 0.5,
            sharpness: 0.3
        }
    }

}

#[derive(Debug)]
pub enum MaterialType {
    Metal,
    Wood,
    Bone,
}