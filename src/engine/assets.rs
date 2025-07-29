use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex, MutexGuard}};

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::engine::geometry::Size2D;

static ASSETS: LazyLock<Mutex<Assets>> = LazyLock::new(|| Mutex::new(Assets::new()));

pub(crate) fn assets() -> MutexGuard<'static, Assets> {
    ASSETS.lock().unwrap()
}

pub(crate) struct Assets {
    images: HashMap<String, Arc<Image>>,
    image_sheets: HashMap<String, Arc<ImageSheet>>,
}

impl Assets {

    fn new() -> Self {
        Self {
            images: HashMap::new(),
            image_sheets: HashMap::new()
        }
    }

    pub(crate) fn reload_all(&mut self) {
        self.images.clear();
        self.image_sheets.clear();
    }

    pub(crate) fn image(&mut self, path: &str) -> Arc<Image> {
        let key = String::from(path);
        match self.images.get(&key) {
            None => {
                let path = format!("./assets/sprites/{path}");
                let image = ImageReader::open(&path)
                    .expect(&format!("Image not found: {}", path))
                    .decode().unwrap();
                let settings = TextureSettings::new().filter(Filter::Nearest);
                let texture = Texture::from_image(&image.to_rgba8(), &settings);
                let arc = Arc::new(Image {
                    size: Size2D(image.width() as usize, image.height() as usize),
                    texture
                });
                let arc_clone = arc.clone();
                self.images.insert(key, arc);
                return arc_clone
            },
            Some(value) => value.clone()
        }
    }

    pub(crate) fn image_sheet(&mut self, path: &str, size: Size2D) -> Arc<ImageSheet> {
        let key = String::from(path);
        match self.image_sheets.get(&key) {
            None => {
                let path = format!("./assets/sprites/{}", path);
                let image = ImageReader::open(&path).unwrap().decode().unwrap();
                let settings = TextureSettings::new().filter(Filter::Nearest);
                let mut textures = Vec::new();
                let tiles_x = image.width() / size.0 as u32;
                let tiles_y = image.height() / size.1 as u32;
                for y in 0..tiles_y {
                    for x in 0..tiles_x {
                        let tile = image.crop_imm(x * size.0 as u32, y * size.1 as u32, size.0 as u32, size.1 as u32).to_rgba8();
                        // TODO: Subimage works with references. Maybe it's better?
                        //let tile = image.sub_image(x, y, params.tile_size.0 as u32, params.tile_size.1 as u32);
                        textures.push(Texture::from_image(&tile, &settings));
                    }
                }
                let arc = Arc::new(ImageSheet {
                    tile_size: size,
                    textures,
                });
                let arc_clone = arc.clone();
                self.image_sheets.insert(key, arc);
                return arc_clone
            },
            Some(value) => value.clone()
        }
    }

}

pub(crate) struct Image {
    pub(crate) size: Size2D,
    pub(crate) texture: Texture
}

pub(crate) struct ImageSheet {
    pub(crate) tile_size: Size2D,
    pub(crate) textures: Vec<Texture>
}

impl ImageSheet {
    pub(crate) fn len(&self) -> usize {
        return self.textures.len()
    }

    pub(crate) fn get(&self, i: usize) -> Option<&Texture> {
        return self.textures.get(i)
    }
}