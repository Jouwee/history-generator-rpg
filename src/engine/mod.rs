use image::Rgba;

pub(crate) mod animation;
pub(crate) mod assets;
pub(crate) mod audio;
pub(crate) mod debug;
pub(crate) mod geometry;
pub(crate) mod gui;
pub(crate) mod input;
pub(crate) mod layered_dualgrid_tilemap;
pub(crate) mod pallete_sprite;
pub(crate) mod render;
pub(crate) mod spritesheet;
pub(crate) mod scene;
pub(crate) mod sprite;
pub(crate) mod tilemap;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub(crate) struct Point2D(pub(crate) usize, pub(crate) usize);

impl Point2D {

    pub(crate) fn dist_squared(&self, another: &Point2D) -> f32 {
        let x = another.0 as f32 - self.0 as f32;
        let y = another.1 as f32 - self.1 as f32;
        return x*x + y*y
    }

}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) struct Color {
    pub(crate) r: f32,
    pub(crate) g: f32,
    pub(crate) b: f32,
    pub(crate) a: f32,
}

impl Color {

    pub(crate) fn rgb(rgb: [f32; 3]) -> Color {
        Color {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 1.0,
        }
    }

    pub(crate) fn from_hex(hex: &str) -> Color {
        assert!(hex.len() == 6 || hex.len() == 8, "A hex color must be 6 or 8 characters long. I got {}", hex);
        let r = u8::from_str_radix(&hex[0..2], 16).expect("Wrong red channel");
        let g = u8::from_str_radix(&hex[2..4], 16).expect("Wrong green channel");
        let b = u8::from_str_radix(&hex[4..6], 16).expect("Wrong blue channel");
        let mut a: u8 = 255;
        if hex.len() == 8 {
            a = u8::from_str_radix(&hex[6..8], 16).expect("Wrong alpha channel");
        }
        return Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0
        };
    }

    pub(crate) fn f32_arr(&self) -> [f32; 4] {
        return [self.r, self.g, self.b, self.a]
    }

    pub(crate) fn to_rgba8(&self) -> Rgba<u8> {
        Rgba::<u8>([
            (self.r * 255.) as u8,
            (self.g * 255.) as u8,
            (self.b * 255.) as u8,
            (self.a * 255.) as u8,
        ])
    }

}

pub(crate) enum Palette {
    Green,
    Red,
    Gray
}

impl Palette {

    pub(crate) fn color(&self) -> Color {
        match self {
            Palette::Green => Color::from_hex("3c502d"),
            Palette::Red => Color::from_hex("882309"),
            Palette::Gray => Color::from_hex("b2b1c4")
        }
    }

}