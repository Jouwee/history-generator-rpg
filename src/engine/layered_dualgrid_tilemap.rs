
use crate::{engine::assets::{assets, GetSprite, ImageSheetAsset, ImageSheetSprite}, globals::perf::perf};

use super::{render::RenderContext};

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

pub(crate) struct LayeredDualgridTilemap {
    tiles: Vec<Option<usize>>,
    collapsed_tiles: Vec<Vec<ImageSheetSprite>>,
    tileset: LayeredDualgridTileset,
    width: usize,
    height: usize,
    cell_width: usize,
    cell_height: usize
}

impl LayeredDualgridTilemap {

    pub(crate) fn new(tileset: LayeredDualgridTileset, width: usize, height: usize, cell_width: usize, cell_height: usize) -> LayeredDualgridTilemap {
        let mut collapsed_tiles = Vec::new();
        for _ in 0..(width * height) {
            collapsed_tiles.push(Vec::new());
        }
        LayeredDualgridTilemap {
            tiles: vec![None; width * height],
            collapsed_tiles,
            tileset,
            width,
            height,
            cell_width,
            cell_height
        }
    }

    pub(crate) fn tiles(&self) -> &Vec<Option<usize>> {
        return &self.tiles
    }

    pub(crate) fn tile(&self, x: usize, y: usize) -> Option<usize> {
        return self.tiles[x + y * self.width]
    }

    pub(crate) fn set_tile(&mut self, x: usize, y: usize, tile: usize) {
        self.tiles[x + y * self.width] = Some(tile);
        self.collapse_tile(x, y);
        if x > 0 {
            self.collapse_tile(x - 1, y);
        }
        if y > 0 {
            self.collapse_tile(x, y - 1);
        }
        if x > 0 &&  y > 0 {
            self.collapse_tile(x - 1, y - 1);
        }
    }

    pub(crate) fn render(&self, ctx: &mut RenderContext) {
        perf().start("dualgrid_tilemap");
        let hw = self.cell_width / 2;
        let hh = self.cell_height / 2;
        let cull_start = [
            (ctx.camera_rect[0] / self.cell_width as f64 - 1.).max(0.) as usize,
            (ctx.camera_rect[1] / self.cell_height as f64 - 1.).max(0.) as usize
        ];
        let cull_limit = [
            1 + cull_start[0] + ctx.camera_rect[2] as usize / self.cell_width,
            1 + cull_start[1] + ctx.camera_rect[3] as usize / self.cell_height
        ];
        let x_range = (cull_start[0])..(self.width.min(cull_limit[0] + 2));
        let y_range = (cull_start[1])..(self.height.min(cull_limit[1] + 2));

        for layer in 0..4 {
            for y in y_range.clone() {
                let py = (y * self.cell_height + hh) as f64;
                for x in x_range.clone() {
                    let px = (x * self.cell_width + hw) as f64;
                    let transform = ctx.at(px, py);
                    if let Some(sprite) = &self.collapsed_tiles[x + y * self.width].get(layer) {
                        sprite.draw(transform, ctx.gl);
                    }
                }
            }
        }
        perf().end("dualgrid_tilemap");
    }

    pub(crate) fn collapse_tile(&mut self, x: usize, y: usize) {
        let i = x + y * self.width;
        if let Some(tile_idx) = self.tiles[x + y * self.width] {
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
                let asset = &self.tileset.tiles[grid[0]].textures;
                let sprite = assets().image_sheet(&asset.path, asset.tile_size).sprite(IDX_FULL);
                if let Some(sprite) = sprite {
                    self.collapsed_tiles[i] = vec!(sprite);
                }
                return
            }
            let mut layers = grid.clone().map(|i| (i, &self.tileset.tiles[i]));
            layers.sort_by(|a, b| a.1.layer.cmp(&b.1.layer));

            let last_layer = layers[0];

            let mut tile = Vec::new();

            let asset = &self.tileset.tiles[layers[0].0].textures;
            let sheet = assets().image_sheet(&asset.path, asset.tile_size);
            // Base
            tile.push(sheet.sprite(IDX_FULL).unwrap());

            for i in 1..4 {
                let layer = layers[i];
                if layer.0 == last_layer.0 {
                    continue;
                }
                let asset = &self.tileset.tiles[layer.0].textures;
                let sheet = assets().image_sheet(&asset.path, asset.tile_size);

                let b_grid = grid.map(|i| i == layer.0);
                match b_grid {
                    [false, false, false, false] => tile.push(sheet.sprite(IDX_EMPTY).unwrap()),
                    [false, false, false, true] => tile.push(sheet.sprite(IDX_BR).unwrap()),
                    [false, false, true, false] => tile.push(sheet.sprite(IDX_BL).unwrap()),
                    [false, false, true, true] => tile.push(sheet.sprite(IDX_BR_BL).unwrap()),
                    [false, true, false, false] => tile.push(sheet.sprite(IDX_TR).unwrap()),
                    [false, true, false, true] => tile.push(sheet.sprite(IDX_TR_BR).unwrap()),
                    [false, true, true, false] => tile.push(sheet.sprite(IDX_TR_BL).unwrap()),
                    [false, true, true, true] => tile.push(sheet.sprite(IDX_TR_BL_BR).unwrap()),
                    [true, false, false, false] => tile.push(sheet.sprite(IDX_TL).unwrap()),
                    [true, false, false, true] => tile.push(sheet.sprite(IDX_TL_BR).unwrap()),
                    [true, false, true, false] => tile.push(sheet.sprite(IDX_TL_BL).unwrap()),
                    [true, false, true, true] => tile.push(sheet.sprite(IDX_TL_BL_BR).unwrap()),
                    [true, true, false, false] => tile.push(sheet.sprite(IDX_TL_TR).unwrap()),
                    [true, true, false, true] => tile.push(sheet.sprite(IDX_TL_TR_BR).unwrap()),
                    [true, true, true, false] => tile.push(sheet.sprite(IDX_TL_TR_BL).unwrap()),
                    [true, true, true, true] => tile.push(sheet.sprite(IDX_FULL).unwrap()),
                };
                
            }
            
            self.collapsed_tiles[i] = tile;
        }
    }

}

pub(crate) struct LayeredDualgridTileset {
    tiles: Vec<DualgridTile>,
}

impl LayeredDualgridTileset {
    pub(crate) fn new() -> LayeredDualgridTileset {
        LayeredDualgridTileset {
            tiles: Vec::new()
        }
    }

    pub(crate) fn add(&mut self, layer: u16, image: ImageSheetAsset) {
        self.tiles.push(DualgridTile {
            layer,
            textures: image
        })
    }

}

struct DualgridTile {
    layer: u16,
    textures: ImageSheetAsset
}