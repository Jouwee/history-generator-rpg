use geometry::{Coord2, Vec2};

pub mod assets;
pub mod debug;
pub mod geometry;
pub mod gui;
pub mod render;
pub mod spritesheet;
pub mod scene;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Point2D(pub usize, pub usize);

impl Point2D {

    pub fn dist_squared(&self, another: &Point2D) -> f32 {
        let x = another.0 as f32 - self.0 as f32;
        let y = another.1 as f32 - self.1 as f32;
        return x*x + y*y
    }

    pub fn vec_between(&self, another: &Point2D) -> Vec2 {
        Vec2::xy(self.0 as f32, self.1 as f32) - Vec2::xy(another.0 as f32, another.1 as f32)
    }

    pub fn to_coord(&self) -> Coord2 {
        Coord2 { x: self.0 as i32, y: self.1 as i32 }
    }

}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {

    pub fn rgb(rgb: [f32; 3]) -> Color {
        Color {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 1.0,
        }
    }

    pub fn from_hex(hex: &str) -> Color {
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

    pub fn alpha(&self, alpha: f32) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha
        }
    }

    pub fn f32_arr(&self) -> [f32; 4] {
        return [self.r, self.g, self.b, self.a]
    }
}