use std::collections::HashMap;

use image::ImageReader;
use opengl_graphics::{Filter, GlyphCache, Texture, TextureSettings};

pub struct Assets {
    textures: HashMap<String, Asset<Texture>>,
    fonts: HashMap<String, Asset<GlyphCache<'static>>>
}

impl Assets {

    pub fn new() -> Assets {
        Assets {
            textures: HashMap::new(),
            fonts: HashMap::new()
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
}

pub struct Asset<T> {
    value: T
}