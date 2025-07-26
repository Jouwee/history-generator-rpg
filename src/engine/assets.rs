use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex, MutexGuard}};

use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::engine::geometry::Size2D;

static ASSETS: LazyLock<Mutex<Assets>> = LazyLock::new(|| Mutex::new(Assets::new()));

pub(crate) fn assets() -> MutexGuard<'static, Assets> {
    ASSETS.lock().unwrap()
}

pub(crate) struct Assets {
    images: HashMap<String, Arc<Image>>
}

impl Assets {

    fn new() -> Self {
        Self {
            images: HashMap::new()
        }
    }

    pub(crate) fn reload_all(&mut self) {
        self.images.clear();
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

}

pub(crate) struct Image {
    pub(crate) size: Size2D,
    pub(crate) texture: Texture
}