use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::engine::geometry::Size2D;


#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ImageAsset {
    pub(crate) path: String,
    rotate: ImageRotate
}

impl ImageAsset {
    pub(crate) fn new(path: &str) -> Self {
        Self {
            path: String::from(path),
            rotate: ImageRotate::None
        }
    }

    pub(crate) fn rotate(mut self, rotate: ImageRotate) -> Self {
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
    pub(crate) size: Size2D,
    pub(crate) texture: Texture
}

impl Image {

    pub(crate) fn new(params: &ImageAsset) -> Image {
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
            size: Size2D(image.width() as usize, image.height() as usize),
            texture
        }
    }

}