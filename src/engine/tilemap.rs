use crate::{commons::rng::Rng, GameContext};

use super::{asset::{image::ImageAsset, image_sheet::ImageSheetAsset}, render::RenderContext};

pub(crate) struct TileMap {
    tiles: Vec<usize>,
    tileset: TileSet,
    width: usize,
    height: usize,
    cell_width: usize,
    cell_height: usize
}

impl TileMap {

    pub(crate) fn new(tileset: TileSet, width: usize, height: usize, cell_width: usize, cell_height: usize) -> TileMap {
        TileMap {
            tiles: vec![0; height * width],
            tileset,
            width,
            height,
            cell_width,
            cell_height 
        }
    }

    pub(crate) fn set_tile(&mut self, x: usize, y: usize, tile: usize) {
        self.tiles[(y*self.width) + x] = tile;
    }

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> &Tile {
        let idx = (y * self.width) + x;
        let tile_i = self.tiles[idx];
        &self.tileset.tiles[tile_i]
    }

    pub(crate) fn get_tile_idx(&self, x: usize, y: usize) -> usize {
        let idx = (y * self.width) + x;
        return self.tiles[idx]
    }

    pub(crate) fn render<F>(&self, ctx: &mut RenderContext, game_ctx: &mut GameContext, mut z_order_render: F) where F: FnMut(&mut RenderContext, &mut GameContext, usize, usize) -> () {
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = (y * self.width) + x;
                let tile_i = self.tiles[idx];
                match &self.tileset.tiles[tile_i] {
                    Tile::Empty => (),
                    Tile::SingleTile(tile) => {
                        let image = game_ctx.assets.image(&tile.image);
                        let pos = [
                            x as f64 * self.cell_width as f64 - (image.size.x() as f64 - self.cell_width as f64) / 2.,
                            y as f64 * self.cell_height as f64 - (image.size.y() as f64 - self.cell_height as f64),
                        ];
                        ctx.texture_ref(&image.texture, pos);
                    },
                    Tile::TileRandom(tile) => {
                        let sheet = game_ctx.assets.image_sheet(&tile.image_sheet);
                        let pos = [
                            x as f64 * self.cell_width as f64 - (sheet.tile_size.x() as f64 - self.cell_width as f64) / 2.,
                            y as f64 * self.cell_height as f64 - (sheet.tile_size.y() as f64 - self.cell_height as f64),
                        ];
                        let mut rng = Rng::new(idx as u32);
                        let i = rng.randu_range(0, sheet.len());
                        ctx.texture_ref(&sheet.get(i).unwrap(), pos);
                    },
                    Tile::T16Subset(tile) => {
                        let sheet = game_ctx.assets.image_sheet(&tile.image_sheet);
                        let pos = [
                            x as f64 * self.cell_width as f64 - (sheet.tile_size.x() as f64 - self.cell_width as f64) / 2.,
                            y as f64 * self.cell_height as f64 - (sheet.tile_size.y() as f64 - self.cell_height as f64),
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
                        let subtile_i = match (u, d, l, r) {
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
                        ctx.texture_ref(sheet.get(subtile_i).unwrap(), pos);
                    }
                }
                z_order_render(ctx, game_ctx, x, y);
            }
        }
    }

}

pub(crate) struct TileSet {
    tiles: Vec<Tile>
}

impl TileSet {
    pub(crate) fn new() -> TileSet {
        TileSet {
            tiles: vec!(Tile::Empty)
        }
    }

    pub(crate) fn add(&mut self, tile: Tile) {
        self.tiles.push(tile);
    }
}

#[derive(Clone)]
pub(crate) enum Tile {
    Empty,
    SingleTile(TileSingle),
    TileRandom(TileRandom),
    T16Subset(Tile16Subset)
}

#[derive(Clone)]
pub(crate) struct TileSingle {
    image: ImageAsset
}

impl TileSingle {
    pub(crate) fn new(image: ImageAsset) -> Self {
        Self {
            image
        }
    }
}

#[derive(Clone)]
pub(crate) struct TileRandom {
    image_sheet: ImageSheetAsset
}

impl TileRandom {
    pub(crate) fn new(image_sheet: ImageSheetAsset) -> Self {
        Self { image_sheet }
    }
}

#[derive(Clone)]
pub(crate) struct Tile16Subset {
    image_sheet: ImageSheetAsset
}

impl Tile16Subset {
    pub(crate) fn new(image_sheet: ImageSheetAsset) -> Self {
        Self {
            image_sheet
        }
    }
}