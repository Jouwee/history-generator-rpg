use std::{collections::HashMap, sync::{Arc, LazyLock, Mutex, MutexGuard}};

use graphics::{CharacterCache, DrawState, Image as GlImage, ImageSize, Transformed};
use image::ImageReader;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, Texture, TextureSettings};

use crate::{engine::{geometry::Size2D, render::RenderContext}, warn};

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
                let mut map = Vec::new();
                let tiles_x = image.width() / size.0 as u32;
                let tiles_y = image.height() / size.1 as u32;
                for y in 0..tiles_y {
                    for x in 0..tiles_x {
                        let tile = image.crop_imm(x * size.0 as u32, y * size.1 as u32, size.0 as u32, size.1 as u32).to_rgba8();
                        textures.push(Texture::from_image(&tile, &settings));
                        map.push([x as f64 * size.0 as f64, y as f64 * size.1 as f64, size.0 as f64, size.1 as f64]);
                    }
                }
                let arc = Arc::new(ImageSheet {
                    texture: Texture::from_image(&image.to_rgba8(), &settings),
                    map,
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

    pub(crate) fn asset_count(&self) -> usize {
        return self.images.len() + self.image_sheets.len() + self.fonts.len();
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
    pub(crate) texture: Texture,
    pub(crate) map: Vec<[f64;4]>,
    pub(crate) textures: Vec<Texture>
}

impl ImageSheet {
    pub(crate) fn len(&self) -> usize {
        return self.textures.len()
    }

    pub(crate) fn get(&self, i: usize) -> Option<&Texture> {
        return self.textures.get(i)
    }

    /// Draw this spritesheet as a scalable background (only 3x3 or 1x3)
    pub(crate) fn draw_as_scalable(&self, rect: [f64; 4], ctx: &mut RenderContext) {
        let w = self.tile_size.x() as f64;
        let h = self.tile_size.y() as f64;
        let image = GlImage::new();
        let draw_state = ctx.context.draw_state;
        match self.len() {
            9 => {
                // Body
                let transform = ctx.context.transform.trans(rect[0] + w, rect[1] + h).scale((rect[2] - w * 2.) / w, (rect[3] - h * 2.) / h);
                image.src_rect(self.map[4]).draw(&self.texture, &draw_state, transform, ctx.gl);
                // Borders
                let transform = ctx.context.transform.trans(rect[0] + w, rect[1]).scale((rect[2] - w) / w, 1.);
                image.src_rect(self.map[1]).draw(&self.texture, &draw_state, transform, ctx.gl);
                let transform = ctx.context.transform.trans(rect[0] + w, rect[1] + rect[3] - h).scale((rect[2] - w) / w, 1.);
                image.src_rect(self.map[7]).draw(&self.texture, &draw_state, transform, ctx.gl);
                let transform = ctx.context.transform.trans(rect[0], rect[1] + h).scale(1., (rect[3] - h) / h);
                image.src_rect(self.map[3]).draw(&self.texture, &draw_state, transform, ctx.gl);
                let transform = ctx.context.transform.trans(rect[0] + rect[2] - w, rect[1] + h).scale(1., (rect[3] - h) / h);
                image.src_rect(self.map[5]).draw(&self.texture, &draw_state, transform, ctx.gl);
                // Corners
                let transform = ctx.context.transform.trans(rect[0], rect[1]);
                image.src_rect(self.map[0]).draw(&self.texture, &draw_state, transform, ctx.gl);
                let transform = ctx.context.transform.trans(rect[0], rect[1] + rect[3] - h);
                image.src_rect(self.map[6]).draw(&self.texture, &draw_state, transform, ctx.gl);
                let transform = ctx.context.transform.trans(rect[0] + rect[2] - w, rect[1]);
                image.src_rect(self.map[2]).draw(&self.texture, &draw_state, transform, ctx.gl);
                let transform = ctx.context.transform.trans(rect[0] + rect[2] - w, rect[1] + rect[3] - h);
                image.src_rect(self.map[8]).draw(&self.texture, &draw_state, transform, ctx.gl);
            },
            3 => {
                let size = self.texture.get_size();
                // Horizontal
                if size.0 > self.tile_size.0 as u32 {
                    // Left
                    let transform = ctx.context.transform.trans(rect[0], rect[1]);
                    image.src_rect(self.map[0]).draw(&self.texture, &draw_state, transform, ctx.gl);
                    // Center
                    let transform = ctx.context.transform.trans(rect[0] + w, rect[1]).scale((rect[2] - w * 2.) / w, 1.);
                    image.src_rect(self.map[1]).draw(&self.texture, &draw_state, transform, ctx.gl);
                    // Right
                    let transform = ctx.context.transform.trans(rect[0] + rect[2] - w, rect[1]);
                    image.src_rect(self.map[2]).draw(&self.texture, &draw_state, transform, ctx.gl);
                } else { // Vertical
                    // Top
                    let transform = ctx.context.transform.trans(rect[0], rect[1]);
                    image.src_rect(self.map[0]).draw(&self.texture, &draw_state, transform, ctx.gl);
                    // Center
                    let transform = ctx.context.transform.trans(rect[0], rect[1] + h).scale(1., (rect[3] - h * 2.) / h);
                    image.src_rect(self.map[1]).draw(&self.texture, &draw_state, transform, ctx.gl);
                    // Bottom
                    let transform = ctx.context.transform.trans(rect[0], rect[1] + rect[3] - h);
                    image.src_rect(self.map[2]).draw(&self.texture, &draw_state, transform, ctx.gl);
                }
            },
            _ => warn!("Can't draw sprite as scalable with size {}", self.len())
        }
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
            rect: *self.map.get(i).unwrap()
        })
    }
}

pub(crate) struct ImageSheetSprite {
    image: Arc<ImageSheet>,
    rect: [f64; 4]
}

impl ImageSheetSprite {

    pub(crate) fn draw(&self, transform: [[f64; 3]; 2], gl: &mut GlGraphics) {
        GlImage::new().src_rect(self.rect).draw(&self.image.texture, &DrawState::default(), transform, gl);
    }

    pub(crate) fn draw_colored(&self, transform: [[f64; 3]; 2], color: [f32; 4], gl: &mut GlGraphics) {
        GlImage::new().color(color).src_rect(self.rect).draw(&self.image.texture, &DrawState::default(), transform, gl);
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