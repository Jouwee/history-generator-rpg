use image::DynamicImage;
use opengl_graphics::{Filter, Texture, TextureSettings};

use super::render::RenderContext;

pub struct TileMap {
    tiles: Vec<usize>,
    tileset: TileSet,
    width: usize,
    height: usize,
    cell_width: usize,
    cell_height: usize
}

impl TileMap {

    pub fn new(tileset: TileSet, width: usize, height: usize, cell_width: usize, cell_height: usize) -> TileMap {
        TileMap {
            tiles: vec![0; height * width],
            tileset,
            width,
            height,
            cell_width,
            cell_height 
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: usize) {
        self.tiles[(y*self.width) + x] = tile;
    }

    pub fn get_tile(&self, x: usize, y: usize) -> &Tile {
        let idx = (y * self.width) + x;
        let tile_i = self.tiles[idx];
        &self.tileset.tiles[tile_i]
    }

    pub fn render<F>(&self, ctx: &mut RenderContext, z_order_render: F) where F: Fn(&mut RenderContext, usize, usize) -> () {
        for x in 0..self.width {
            for y in 0..self.height {
                let idx = (y * self.width) + x;
                let tile_i = self.tiles[idx];
                match &self.tileset.tiles[tile_i] {
                    Tile::Empty => (),
                    Tile::T16Subset(tile) => {
                        let rect = [
                            x as f64 * self.cell_width as f64 - (tile.tile_width - self.cell_width) as f64 / 2.,
                            y as f64 * self.cell_height as f64 - (tile.tile_height - self.cell_height) as f64,
                            self.cell_width as f64,
                            self.cell_height as f64
                        ];
                        let mut u = false;
                        if y > 0 {
                            u = self.tiles[idx - self.width] == tile_i;
                        }
                        let mut d = false;
                        if y < self.height - 1 {
                            d = self.tiles[idx + self.width] == tile_i;
                        }
                        let mut l = false;
                        if x > 0 {
                            l = self.tiles[idx - 1] == tile_i;
                        }
                        let mut r = false;
                        if x < self.width - 1 {
                            r = self.tiles[idx + 1] == tile_i;
                        }
                        let idx = match (u, d, l, r) {
                            (false, false, false, false) => 12,
                            (false, false, false, true) => 13,
                            (false, false, true, false) => 15,
                            (false, false, true, true) => 14,
                            (false, true, false, false) => 0,
                            (false, true, false, true) => 1,
                            (false, true, true, false) => 3,
                            (false, true, true, true) => 2,

                            (true, false, false, false) => 8,
                            (true, false, false, true) => 9,
                            (true, false, true, false) => 11,
                            (true, false, true, true) => 10,
                            (true, true, false, false) => 4,
                            (true, true, false, true) => 5,
                            (true, true, true, false) => 7,
                            (true, true, true, true) => 6,
                        };
                        ctx.texture_ref(tile.textures.get(idx).unwrap(), [rect[0], rect[1]]);
                    }
                }
                z_order_render(ctx, x, y);
            }
        }
    }

}

pub struct TileSet {
    tiles: Vec<Tile>
}

impl TileSet {
    pub fn new() -> TileSet {
        TileSet {
            tiles: vec!(Tile::Empty)
        }
    }

    pub fn add(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }
}

pub enum Tile {
    Empty,
    T16Subset(Tile16Subset)
}

pub struct Tile16Subset {
    tile_width: usize,
    tile_height: usize,
    textures: Vec<Texture>
}

impl Tile16Subset {
    pub fn new(image: DynamicImage, tile_width: usize, tile_height: usize) -> Tile16Subset {
        let mut textures = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let tile = image.crop_imm(x * tile_width as u32, y * tile_height as u32, tile_width as u32, tile_height as u32).to_rgba8();
                let settings = TextureSettings::new().filter(Filter::Nearest);
                textures.push(Texture::from_image(&tile, &settings));
            }
        }
        Tile16Subset {
            tile_width,
            tile_height,
            textures
        }
    }
}