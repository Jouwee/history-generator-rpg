use std::collections::HashMap;

use image::DynamicImage;
use opengl_graphics::{Filter, Texture, TextureSettings};

pub(crate) struct Spritesheet {
    textures: HashMap<(u32, u32), Texture>
}

impl Spritesheet {

    pub(crate) fn new(img: DynamicImage, sprite_size: (u32, u32)) -> Spritesheet {
        let mut sheet = Spritesheet {
            textures: HashMap::new()
        };
        for x in 0..sprite_size.0 {
            for y in 0..sprite_size.1 {
                let sprite = img.crop_imm(x * sprite_size.0, y * sprite_size.1, sprite_size.0, sprite_size.1).to_rgba8();
                let settings = TextureSettings::new().filter(Filter::Nearest);
                let sprite = Texture::from_image(&sprite, &settings);
                sheet.textures.insert((x, y), sprite);
            }
        }
        sheet
    }

    pub(crate) fn sprite(&self, x: u32, y: u32) -> &Texture {
        self.textures.get(&(x, y)).unwrap()
    }
}