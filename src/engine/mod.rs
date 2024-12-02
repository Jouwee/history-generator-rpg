pub mod assets;
pub mod render;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Point2D(pub usize, pub usize);

impl Point2D {

    pub fn dist_squared(&self, another: &Point2D) -> f32 {
        let x = another.0 as f32 - self.0 as f32;
        let y = another.1 as f32 - self.1 as f32;
        return x*x + y*y
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

    pub fn f32_arr(&self) -> [f32; 4] {
        return [self.r, self.g, self.b, self.a]
    }
}