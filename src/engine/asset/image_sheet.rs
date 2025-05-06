use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::engine::geometry::Size2D;


#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub(crate) struct ImageSheetAsset {
    pub(crate) path: String,
    pub(crate) tile_size: Size2D
}

impl ImageSheetAsset {
    pub(crate) fn new(path: &str, tile_size: Size2D) -> Self {
        Self {
            path: String::from(path),
            tile_size
        }
    }
}

pub(crate) struct ImageSheet {
    pub(crate) tile_size: Size2D,
    pub(crate) textures: Vec<Texture>
}

impl ImageSheet {

    pub(crate) fn new(params: &ImageSheetAsset) -> ImageSheet {
        let path = format!("./assets/sprites/{}", params.path);
        let image = ImageReader::open(&path).unwrap().decode().unwrap();
        let settings = TextureSettings::new().filter(Filter::Nearest);
        let mut textures = Vec::new();
        let tiles_x = image.width() / params.tile_size.0 as u32;
        let tiles_y = image.height() / params.tile_size.1 as u32;
        for x in 0..tiles_x {
            for y in 0..tiles_y {
                let tile = image.crop_imm(x * params.tile_size.0 as u32, y * params.tile_size.1 as u32, params.tile_size.0 as u32, params.tile_size.1 as u32).to_rgba8();
                // TODO: Subimage works with references. Maybe it's better?
                //let tile = image.sub_image(x, y, params.tile_size.0 as u32, params.tile_size.1 as u32);
                textures.push(Texture::from_image(&tile, &settings));
            }
        }
        Self {
            tile_size: params.tile_size,
            textures
        }
    }

    pub(crate) fn len(&self) -> usize {
        return self.textures.len()
    }

    pub(crate) fn get(&self, i: usize) -> Option<&Texture> {
        return self.textures.get(i)
    }

}