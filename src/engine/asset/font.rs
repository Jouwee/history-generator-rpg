use graphics::CharacterCache;
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