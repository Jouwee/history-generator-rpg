use std::collections::HashMap;

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use super::spritesheet::Spritesheet;

pub struct Assets {
    textures: HashMap<String, Asset<Texture>>,
    spritesheets: HashMap<String, Asset<Spritesheet>>,
}

impl Assets {

    pub fn new() -> Assets {
        Assets {
            textures: HashMap::new(),
            spritesheets: HashMap::new()
        }
    }

    pub fn texture(&mut self, name: &str) -> &Texture {
        if !self.textures.contains_key(name) {
            let mut path = String::from("./assets/sprites/");
            path.push_str(name);
            let spritesheet = ImageReader::open(path).unwrap().decode().unwrap();
            let settings = TextureSettings::new().filter(Filter::Nearest);
            let texture = Texture::from_image(&spritesheet.to_rgba8(), &settings);
            self.textures.insert(String::from(name), Asset { value: texture });
        }
        &self.textures.get(name).expect(format!("Image {name} does not exist").as_str()).value
    }

    pub fn spritesheet(&mut self, name: &str, size: (u32, u32)) -> &Spritesheet {
        if !self.spritesheets.contains_key(name) {
            let mut path = String::from("./assets/sprites/");
            path.push_str(name);
            let spritesheet = ImageReader::open(path).unwrap().decode().unwrap();
            self.spritesheets.insert(String::from(name), Asset { value: Spritesheet::new(spritesheet, size) });
        }
        &self.spritesheets.get(name).expect(format!("Image {name} does not exist").as_str()).value
    }

}

pub struct Asset<T> {
    value: T
}