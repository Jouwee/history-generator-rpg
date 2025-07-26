use std::collections::HashMap;

use super::{font::{Font, FontAsset}, image_sheet::{ImageSheet, ImageSheetAsset}};

pub(crate) struct Assets {
    image_sheets: HashMap<ImageSheetAsset, Asset<ImageSheet>>,
    fonts: HashMap<FontAsset, Asset<Font>>,
}

impl Assets {

    pub(crate) fn new() -> Assets {
        Assets {
            image_sheets: HashMap::new(),
            fonts: HashMap::new(),
        }
    }

    pub(crate) fn image_sheet(&mut self, params: &ImageSheetAsset) -> &ImageSheet {
        if !self.image_sheets.contains_key(&params) {
            let image = ImageSheet::new(&params);
            self.image_sheets.insert(params.clone(), Asset { value: image });
        }
        &self.image_sheets.get(&params).expect(format!("Image {} does not exist", params.path).as_str()).value
    }

    pub(crate) fn font(&mut self, params: &FontAsset) -> &mut Font {
        if !self.fonts.contains_key(&params) {
            let font = Font::new(&params);
            self.fonts.insert(params.clone(), Asset { value: font });
        }
        &mut self.fonts.get_mut(&params).expect(format!("Font {} does not exist", params.path).as_str()).value
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

struct Asset<T> {
    value: T
}