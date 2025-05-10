use std::collections::HashMap;

use super::{image::{Image, ImageAsset}, image_sheet::{ImageSheet, ImageSheetAsset}};

pub(crate) struct Assets {
    images: HashMap<ImageAsset, Asset<Image>>,
    image_sheets: HashMap<ImageSheetAsset, Asset<ImageSheet>>
}

impl Assets {

    pub(crate) fn new() -> Assets {
        Assets {
            images: HashMap::new(),
            image_sheets: HashMap::new(),
        }
    }

    pub(crate) fn image(&mut self, params: &ImageAsset) -> &Image {
        if !self.images.contains_key(&params) {
            let image = Image::new(&params);
            self.images.insert(params.clone(), Asset { value: image });
        }
        &self.images.get(&params).expect(format!("Image {} does not exist", params.path).as_str()).value
    }

    pub(crate) fn image_sheet(&mut self, params: &ImageSheetAsset) -> &ImageSheet {
        if !self.image_sheets.contains_key(&params) {
            let image = ImageSheet::new(&params);
            self.image_sheets.insert(params.clone(), Asset { value: image });
        }
        &self.image_sheets.get(&params).expect(format!("Image {} does not exist", params.path).as_str()).value
    }

}

struct Asset<T> {
    value: T
}