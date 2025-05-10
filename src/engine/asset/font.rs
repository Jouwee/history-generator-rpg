use std::cell::RefCell;

use opengl_graphics::GlyphCache;
use opengl_graphics::{Filter, TextureSettings};


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

}