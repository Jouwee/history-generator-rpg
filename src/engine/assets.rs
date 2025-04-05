use std::collections::HashMap;

use image::{DynamicImage, ImageReader};
use opengl_graphics::{Filter, Texture, TextureSettings};

use super::spritesheet::Spritesheet;

pub struct Assets {
    images: HashMap<ImageParams, Asset<Image>>
}

impl Assets {

    pub fn new() -> Assets {
        Assets { images: HashMap::new() }
    }

    pub fn image(&mut self, params: ImageParams) -> &Image {
        if !self.images.contains_key(&params) {
            let image = Image::new(&params);
            self.images.insert(params.clone(), Asset { value: image });
        }
        &self.images.get(&params).expect(format!("Image {} does not exist", params.path).as_str()).value
    }
}

struct Asset<T> {
    value: T
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct ImageParams {
    path: String,
    rotate: ImageRotate
}

impl ImageParams {
    pub fn new(path: &str) -> ImageParams {
        ImageParams {
            path: String::from(path),
            rotate: ImageRotate::None
        }
    }

    pub fn rotate(mut self, rotate: ImageRotate) -> ImageParams {
        self.rotate = rotate;
        return self
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum ImageRotate {
    None,
    R90,
    R180,
    R270
}

pub struct Image {
    pub size: (u32, u32),
    pub image: DynamicImage,
    pub texture: Texture
}

impl Image {

    pub fn new(params: &ImageParams) -> Image {
        let path = format!("./assets/sprites/{}", params.path);
        let image = ImageReader::open(&path).unwrap().decode().unwrap();
        let image = match params.rotate {
            ImageRotate::None => image,
            ImageRotate::R90 => image.rotate90(),
            ImageRotate::R180 => image.rotate180(),
            ImageRotate::R270 => image.rotate270(),
        };
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let texture = Texture::from_image(&image.to_rgba8(), &settings);
        Self {
            size: (image.width(), image.height()),
            image,
            texture
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