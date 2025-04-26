use std::collections::HashMap;

use image::{DynamicImage, ImageReader};
use opengl_graphics::{Filter, Texture, TextureSettings};

use super::spritesheet::Spritesheet;

pub(crate) struct Assets {
    images: HashMap<ImageParams, Asset<Image>>
}

impl Assets {

    pub(crate) fn new() -> Assets {
        Assets { images: HashMap::new() }
    }

    pub(crate) fn image(&mut self, params: ImageParams) -> &Image {
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
pub(crate) struct ImageParams {
    path: String,
    rotate: ImageRotate
}

impl ImageParams {
    pub(crate) fn new(path: &str) -> ImageParams {
        ImageParams {
            path: String::from(path),
            rotate: ImageRotate::None
        }
    }

    pub(crate) fn rotate(mut self, rotate: ImageRotate) -> ImageParams {
        self.rotate = rotate;
        return self
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub(crate) enum ImageRotate {
    None,
    R90,
    R180,
    R270
}

pub(crate) struct Image {
    pub(crate) size: (u32, u32),
    pub(crate) image: DynamicImage,
    pub(crate) texture: Texture
}

impl Image {

    pub(crate) fn new(params: &ImageParams) -> Image {
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

pub(crate) struct OldAssets {
    textures: HashMap<String, OldAsset<Texture>>,
    spritesheets: HashMap<String, OldAsset<Spritesheet>>,
}

impl OldAssets {

    pub(crate) fn new() -> OldAssets {
        OldAssets {
            textures: HashMap::new(),
            spritesheets: HashMap::new()
        }
    }

    pub(crate) fn texture(&mut self, name: &str) -> &Texture {
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

    pub(crate) fn spritesheet(&mut self, name: &str, size: (u32, u32)) -> &Spritesheet {
        if !self.spritesheets.contains_key(name) {
            let mut path = String::from("./assets/sprites/");
            path.push_str(name);
            let spritesheet = ImageReader::open(path).unwrap().decode().unwrap();
            self.spritesheets.insert(String::from(name), OldAsset { value: Spritesheet::new(spritesheet, size) });
        }
        &self.spritesheets.get(name).expect(format!("Image {name} does not exist").as_str()).value
    }

}

pub(crate) struct OldAsset<T> {
    value: T
}