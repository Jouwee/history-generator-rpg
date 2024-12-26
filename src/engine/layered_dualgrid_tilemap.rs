use image::DynamicImage;
use opengl_graphics::{Filter, Texture, TextureSettings};

use super::render::RenderContext;

pub struct LayeredDualgridTilemap {
    tiles: Vec<Option<usize>>,
    tileset: LayeredDualgridTileset,
    width: usize,
    height: usize,
    cell_width: usize,
    cell_height: usize
}

const IDX_BL: usize = 0;
const IDX_TR_BR: usize = 1;
const IDX_TL_BL_BR: usize = 2;
const IDX_BR_BL: usize = 3;
const IDX_TL_BR: usize = 4;
const IDX_TR_BL_BR: usize = 5;
const IDX_FULL: usize = 6;
const IDX_TL_TR_BL: usize = 7;
const IDX_TR: usize = 8;
const IDX_TL_TR: usize = 9;
const IDX_TL_TR_BR: usize = 10;
const IDX_TL_BL: usize = 11;
const IDX_EMPTY: usize = 12;
const IDX_BR: usize = 13;
const IDX_TR_BL: usize = 14;
const IDX_TL: usize = 15;


impl LayeredDualgridTilemap {

    pub fn new(tileset: LayeredDualgridTileset, width: usize, height: usize, cell_width: usize, cell_height: usize) -> LayeredDualgridTilemap {
        LayeredDualgridTilemap {
            tiles: vec![None; width * height],
            tileset,
            width,
            height,
            cell_width,
            cell_height
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: usize) {
        self.tiles[x + y * self.width] = Some(tile);
    }

    pub fn render(&self, ctx: &mut RenderContext) {
        let hw = self.cell_width as f64 / 2.;
        let hh = self.cell_height as f64 / 2.;
        for y in 0..self.width {
            for x in 0..self.height {
                if let Some(tile_idx) = self.tiles[x + y * self.width] {
                    let pos = [(x * self.cell_width) as f64 + hw, (y * self.cell_height) as f64 + hh];
                    let mut grid = [tile_idx; 4];
                    if x < self.width - 1 {
                        if let Some(t) = self.tiles[(x + 1) + y * self.width] {
                            grid[1] = t;
                        }
                    }
                    if y < self.height - 1 {
                        if let Some(t) = self.tiles[x + (y + 1) * self.width] {
                            grid[2] = t;
                        }
                    }
                    if x < self.width - 1 && y < self.height - 1 {
                        if let Some(t) = self.tiles[(x + 1) + (y + 1) * self.width] {
                            grid[3] = t;
                        }
                    }
                    if grid[0] == grid[1] && grid[0] == grid[2] && grid[0] == grid[3] {
                        ctx.texture_ref(&self.tileset.tiles[grid[0]].textures[IDX_FULL], pos);
                        continue
                    }
                    let mut layers = grid.clone().map(|i| (i, &self.tileset.tiles[i]));
                    layers.sort_by(|a, b| a.1.layer.cmp(&b.1.layer));

                    let last_layer = layers[0];
                    // Base
                    ctx.texture_ref(&layers[0].1.textures[IDX_FULL], pos);

                    for i in 1..4 {
                        let layer = layers[i];
                        if layer.0 == last_layer.0 {
                            continue;
                        }
                        let b_grid = grid.map(|i| i == layer.0);
                        let texture = match b_grid {
                            [false, false, false, false] => &layer.1.textures[IDX_EMPTY],
                            [false, false, false, true] => &layer.1.textures[IDX_BR],
                            [false, false, true, false] => &layer.1.textures[IDX_BL],
                            [false, false, true, true] => &layer.1.textures[IDX_BR_BL],
                            [false, true, false, false] => &layer.1.textures[IDX_TR],
                            [false, true, false, true] => &layer.1.textures[IDX_TR_BR],
                            [false, true, true, false] => &layer.1.textures[IDX_TR_BL],
                            [false, true, true, true] => &layer.1.textures[IDX_TR_BL_BR],
                            [true, false, false, false] => &layer.1.textures[IDX_TL],
                            [true, false, false, true] => &layer.1.textures[IDX_TL_BR],
                            [true, false, true, false] => &layer.1.textures[IDX_TL_BL],
                            [true, false, true, true] => &layer.1.textures[IDX_TL_BL_BR],
                            [true, true, false, false] => &layer.1.textures[IDX_TL_TR],
                            [true, true, false, true] => &layer.1.textures[IDX_TL_TR_BR],
                            [true, true, true, false] => &layer.1.textures[IDX_TL_TR_BL],
                            [true, true, true, true] => &layer.1.textures[IDX_FULL],
                        };
                        ctx.texture_ref(texture, pos);
                    }
                }
            }
        }
    }

}

pub struct LayeredDualgridTileset {
    tiles: Vec<DualgridTile>,
}

impl LayeredDualgridTileset {
    pub fn new() -> LayeredDualgridTileset {
        LayeredDualgridTileset {
            tiles: Vec::new()
        }
    }

    pub fn add(&mut self, layer: u16, image: DynamicImage, tile_width: u32, tile_height: u32) {
        let mut textures = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                let tile = image.crop_imm(x * tile_width, y * tile_height, tile_width, tile_height).to_rgba8();
                let settings = TextureSettings::new().filter(Filter::Nearest);
                textures.push(Texture::from_image(&tile, &settings));
            }
        }
        self.tiles.push(DualgridTile {
            layer,
            textures
        })
    }

}

struct DualgridTile {
    layer: u16,
    textures: Vec<Texture>
}