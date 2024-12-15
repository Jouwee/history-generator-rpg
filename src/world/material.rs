#[derive(Debug)]
pub struct Material {
    name: String,
    material_type: MaterialType
}

impl Material {

    pub fn new_metal(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Metal
        }
    }

    pub fn new_wood(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Wood
        }
    }

    pub fn new_bone(name: &str) -> Material {
        Material {
            name: name.to_string(),
            material_type: MaterialType::Bone
        }
    }

}

#[derive(Debug)]
pub enum MaterialType {
    Metal,
    Wood,
    Bone,
}