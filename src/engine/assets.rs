use std::collections::HashMap;

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use super::spritesheet::Spritesheet;

pub struct Assets {
    images: HashMap<String, Asset<Image>>
}

impl Assets {

    pub fn new() -> Assets {
        Assets { images: HashMap::new() }
    }

    pub fn image(&mut self, path: &str) -> &Image {
        if !self.images.contains_key(path) {
            let image = Image::new(path);
            self.images.insert(String::from(path), Asset { value: image });
        }
        &self.images.get(path).expect(format!("Image {path} does not exist").as_str()).value
    }
}

struct Asset<T> {
    value: T
}

pub struct Image {
    pub size: (u32, u32),
    pub texture: Texture
}

impl Image {

    pub fn new(path: &str) -> Image {
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let path = format!("./assets/sprites/{}", path.to_string());
        let image = ImageReader::open(&path).unwrap().decode().unwrap();
        Self {
            size: (image.width(), image.height()),
            texture: Texture::from_image(&image.to_rgba8(), &settings)
        }
    }

}

/* ------------------------------------------- */

pub struct OldAssets {
    textures: HashMap<String, OldAsset<Texture>>,
    spritesheets: HashMap<String, OldAsset<Spritesheet>>,
}

impl OldAssets {

    pub fn new() -> OldAssets {
        OldAssets {
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
            self.textures.insert(String::from(name), OldAsset { value: texture });
        }
        &self.textures.get(name).expect(format!("Image {name} does not exist").as_str()).value
    }

    pub fn spritesheet(&mut self, name: &str, size: (u32, u32)) -> &Spritesheet {
        if !self.spritesheets.contains_key(name) {
            let mut path = String::from("./assets/sprites/");
            path.push_str(name);
            let spritesheet = ImageReader::open(path).unwrap().decode().unwrap();
            self.spritesheets.insert(String::from(name), OldAsset { value: Spritesheet::new(spritesheet, size) });
        }
        &self.spritesheets.get(name).expect(format!("Image {name} does not exist").as_str()).value
    }

}

pub struct OldAsset<T> {
    value: T
}