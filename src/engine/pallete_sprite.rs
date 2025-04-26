use std::{collections::HashMap, hash::Hash};

use image::{DynamicImage, Rgba, RgbaImage};
use opengl_graphics::{Filter, Texture, TextureSettings};

use super::Color;

pub(crate) struct PalleteSprite {
    image: DynamicImage
}

impl PalleteSprite {
    pub(crate) fn new(image: DynamicImage) -> PalleteSprite {
        PalleteSprite { image }
    }

    pub(crate) fn remap(&self, color_map: HashMap<ColorMap, [Color; 4]>) -> Texture {
        let original = self.image.as_rgba8().unwrap();
        let color_map = Self::expand_map(color_map);
        let mut remapped = RgbaImage::new(self.image.width(), self.image.height());
        for x in 0..self.image.width() {
            for y in 0..self.image.height() {
                let color = original[(x, y)];
                if let Some(color) = color_map.get(&color) {
                    remapped[(x, y)] = *color;
                } else {
                    remapped[(x, y)] = color;
                }
            }
        }
        let settings = TextureSettings::new().filter(Filter::Nearest);
        return Texture::from_image(&remapped, &settings)
    }

    fn expand_map(color_map: HashMap<ColorMap, [Color; 4]>) -> HashMap<Rgba<u8>, Rgba<u8>> {
        let mut map = HashMap::new();
        for (color, array) in color_map.iter() {
            match color {
                ColorMap::Blue => {
                    map.insert(Rgba::<u8>([48, 81, 130, 255]), array[0].to_rgba8());
                    map.insert(Rgba::<u8>([65, 146, 195, 255]), array[1].to_rgba8());
                    map.insert(Color::from_hex("61d3e3").to_rgba8(), array[2].to_rgba8());
                    map.insert(Color::from_hex("a2fff3").to_rgba8(), array[3].to_rgba8());
                },
                ColorMap::Red => {
                    map.insert(Color::from_hex("b21030").to_rgba8(), array[0].to_rgba8());
                    map.insert(Color::from_hex("db4161").to_rgba8(), array[1].to_rgba8());
                    map.insert(Color::from_hex("ff61b2").to_rgba8(), array[2].to_rgba8());
                    map.insert(Color::from_hex("ffbaeb").to_rgba8(), array[3].to_rgba8());
                },
                ColorMap::Green => {
                    map.insert(Color::from_hex("386d00").to_rgba8(), array[0].to_rgba8());
                    map.insert(Color::from_hex("49aa10").to_rgba8(), array[1].to_rgba8());
                    map.insert(Color::from_hex("71f341").to_rgba8(), array[2].to_rgba8());
                    map.insert(Color::from_hex("a2f3a2").to_rgba8(), array[3].to_rgba8());
                },
                ColorMap::Yellow => {
                    map.insert(Color::from_hex("495900").to_rgba8(), array[0].to_rgba8());
                    map.insert(Color::from_hex("8a8a00").to_rgba8(), array[1].to_rgba8());
                    map.insert(Color::from_hex("ebd320").to_rgba8(), array[2].to_rgba8());
                    map.insert(Color::from_hex("fff392").to_rgba8(), array[3].to_rgba8());
                }
            }
        }
        return map
    }

}

#[derive(Hash, PartialEq, Eq)]
pub(crate) enum ColorMap {
    Blue,
    Red,
    Green,
    Yellow
}