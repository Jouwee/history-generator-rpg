use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex, MutexGuard}};

use graphics::CharacterCache;
use image::ImageReader;
use opengl_graphics::{Filter, GlyphCache, Texture, TextureSettings};

use crate::engine::geometry::Size2D;

static ASSETS: LazyLock<Mutex<Assets>> = LazyLock::new(|| Mutex::new(Assets::new()));

pub(crate) fn assets() -> MutexGuard<'static, Assets> {
    ASSETS.lock().unwrap()
}

pub(crate) struct Assets {
    images: HashMap<String, Arc<Image>>,
    image_sheets: HashMap<String, Arc<ImageSheet>>,
    fonts: HashMap<FontAsset, Font>,
}

impl Assets {

    fn new() -> Self {
        Self {
            images: HashMap::new(),
            image_sheets: HashMap::new(),
            fonts: HashMap::new()
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

    pub(crate) fn font(&mut self, params: &FontAsset) -> &mut Font {
        if !self.fonts.contains_key(&params) {
            let font = Font::new(&params);
            self.fonts.insert(params.clone(), font);
        }
        self.fonts.get_mut(&params).expect(format!("Font {} does not exist", params.path).as_str())
    }

    pub(crate) fn font_standard_asset() -> FontAsset {
        return FontAsset::new("Everyday_Standard.ttf", 6)
    }

    pub(crate) fn font_standard(&mut self) -> &mut Font {
        return self.font(&Self::font_standard_asset())
    }

    pub(crate) fn font_heading_asset() -> FontAsset {
        return FontAsset::new("Fabled.ttf", 11)
    }

    pub(crate) fn font_heading(&mut self) -> &mut Font {
        return self.font(&Self::font_heading_asset())
    }

}

pub(crate) struct Image {
    pub(crate) size: Size2D,
    pub(crate) texture: Texture
}



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
    pub(crate) fn len(&self) -> usize {
        return self.textures.len()
    }

    pub(crate) fn get(&self, i: usize) -> Option<&Texture> {
        return self.textures.get(i)
    }
}

pub(crate) trait GetSprite {
    fn sprite(&self, i: usize) -> Option<ImageSheetSprite>;
}

impl GetSprite for Arc<ImageSheet> {
    fn sprite(&self, i: usize) -> Option<ImageSheetSprite> {
        if i >= self.len() {
            return None;
        }
        return Some(ImageSheetSprite {
            image: self.clone(),
            index: i
        })
    }
}

pub(crate) struct ImageSheetSprite {
    image: Arc<ImageSheet>,
    index: usize
}

impl ImageSheetSprite {

    pub(crate) fn texture(&self) -> &Texture {
        return self.image.get(self.index).unwrap()
    }

}


#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub(crate) struct FontAsset {
    pub(crate) path: String,
    pub(crate) size: u32,
}

impl FontAsset {

    pub(crate) fn new(path: &str, size: u32) -> Self {
        Self {
            path: String::from(path),
            size
        }
    }

}

pub(crate) struct Font {
    pub(crate) glyphs: GlyphCache<'static>,
    pub(crate) size: u32,
}

impl Font {

    pub(crate) fn new(params: &FontAsset) -> Self {
        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        let path = format!("./assets/fonts/{}", params.path);
        let glyphs = GlyphCache::new(path, (), texture_settings).expect("Could not load font");
        
        Self {
            glyphs,
            size: params.size
        }
    }

    pub(crate) fn width(&mut self, text: &str) -> f64 {
        return self.glyphs.width(self.size, text).unwrap_or(0.);
    }

    pub(crate) fn line_height(&mut self) -> f64 {
        let mut height: f64 = 0.0;
        for ch in ['W', 'q'] {
            let character = self.glyphs.character(self.size, ch);
            if let Ok(character) = character {
                height = height.max(character.atlas_size[1]);
            }
        }
        return height
    }

}