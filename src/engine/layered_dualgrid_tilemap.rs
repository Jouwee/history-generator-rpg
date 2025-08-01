use crate::{engine::assets::ImageSheetAsset, globals::perf::perf};

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
    collapsed_tiles: Vec<Option<Vec<(usize, usize)>>>,
    tileset: LayeredDualgridTileset,
    width: usize,
    height: usize,
    cell_width: usize,
    cell_height: usize
}

impl LayeredDualgridTilemap {

    pub(crate) fn new(tileset: LayeredDualgridTileset, width: usize, height: usize, cell_width: usize, cell_height: usize) -> LayeredDualgridTilemap {
        LayeredDualgridTilemap {
            tiles: vec![None; width * height],
            collapsed_tiles: vec![None; width * height],
            tileset,
            width,
            height,
            cell_width,
            cell_height
        }
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
        for y in y_range {
            for x in x_range.clone() {
                if let Some(tile) = &self.collapsed_tiles[x + y * self.width] {
                    let pos = [(x * self.cell_width + hw) as i32, (y * self.cell_height + hh) as i32];
                    for (tileset, tileset_idx) in tile {
                        ctx.tile(&self.tileset.tiles[*tileset].textures, *tileset_idx, pos);
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
                self.collapsed_tiles[i] = Some(vec!((grid[0], IDX_FULL)));
                return
            }
            let mut layers = grid.clone().map(|i| (i, &self.tileset.tiles[i]));
            layers.sort_by(|a, b| a.1.layer.cmp(&b.1.layer));

            let last_layer = layers[0];

            let mut tile = Vec::new();

            // Base
            tile.push((layers[0].0, IDX_FULL));

            for i in 1..4 {
                let layer = layers[i];
                if layer.0 == last_layer.0 {
                    continue;
                }
                let b_grid = grid.map(|i| i == layer.0);
                match b_grid {
                    [false, false, false, false] => tile.push((layer.0, IDX_EMPTY)),
                    [false, false, false, true] => tile.push((layer.0, IDX_BR)),
                    [false, false, true, false] => tile.push((layer.0, IDX_BL)),
                    [false, false, true, true] => tile.push((layer.0, IDX_BR_BL)),
                    [false, true, false, false] => tile.push((layer.0, IDX_TR)),
                    [false, true, false, true] => tile.push((layer.0, IDX_TR_BR)),
                    [false, true, true, false] => tile.push((layer.0, IDX_TR_BL)),
                    [false, true, true, true] => tile.push((layer.0, IDX_TR_BL_BR)),
                    [true, false, false, false] => tile.push((layer.0, IDX_TL)),
                    [true, false, false, true] => tile.push((layer.0, IDX_TL_BR)),
                    [true, false, true, false] => tile.push((layer.0, IDX_TL_BL)),
                    [true, false, true, true] => tile.push((layer.0, IDX_TL_BL_BR)),
                    [true, true, false, false] => tile.push((layer.0, IDX_TL_TR)),
                    [true, true, false, true] => tile.push((layer.0, IDX_TL_TR_BR)),
                    [true, true, true, false] => tile.push((layer.0, IDX_TL_TR_BL)),
                    [true, true, true, true] => tile.push((layer.0, IDX_FULL)),
                };
                
            }
            self.collapsed_tiles[i] = Some(tile);
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