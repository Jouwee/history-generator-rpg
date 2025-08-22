use std::sync::Arc;

use graphics::{Image, Transformed};

use crate::{commons::rng::Rng, engine::assets::{assets, ImageSheetAsset}, globals::perf::perf, resources::resources::resources, GameContext};

use super::{render::RenderContext};

pub(crate) struct TileMap {
    /// Tile / Shadow?
    tiles: Vec<(usize, bool)>,
    tileset: TileSet,
    width: usize,
    height: usize,
    cell_width: usize,
    cell_height: usize,
    draw_shadows: bool,
}

impl TileMap {

    pub(crate) fn new(tileset: TileSet, width: usize, height: usize, cell_width: usize, cell_height: usize) -> TileMap {
        TileMap {
            tiles: vec![(0, false); height * width],
            tileset,
            width,
            height,
            cell_width,
            cell_height,
            draw_shadows: false,
        }
    }

    pub(crate) fn draw_shadows(mut self) -> Self {
        self.draw_shadows = true;
        self
    }

    pub(crate) fn tiles(&self) -> &Vec<(usize, bool)> {
        return &self.tiles
    }

    pub(crate) fn reset(&mut self) {
        self.tiles = vec![(0, false); self.height * self.width];
    }

    pub(crate) fn set_tile(&mut self, x: usize, y: usize, tile: usize) {
        // SMELL
        let shadow;
        if tile > 0 {
            let resources = resources();
            shadow = resources.object_tiles.try_get(tile - 1).unwrap().casts_shadow;
        } else {
            shadow = false;
        }
        self.tiles[(y*self.width) + x] = (tile, shadow);
    }

    pub(crate) fn set_shadow(&mut self, x: usize, y: usize, shadow: bool) {
        self.tiles[(y*self.width) + x].1 = shadow;
    }

    pub(crate) fn get_tile(&self, x: usize, y: usize) -> &Tile {
        let idx = (y * self.width) + x;
        let tile_i = self.tiles[idx];
        &self.tileset.tiles[tile_i.0]
    }

    pub(crate) fn get_tile_idx(&self, x: usize, y: usize) -> usize {
        let idx = (y * self.width) + x;
        return self.tiles[idx].0
    }

    pub(crate) fn render<F>(&self, ctx: &mut RenderContext, game_ctx: &mut GameContext, mut z_order_render: F) where F: FnMut(&mut RenderContext, &mut GameContext, usize, usize) -> () {
        perf().start("tilemap");
        let cull_start = [
            (ctx.camera_rect[0] / self.cell_width as f64 - 2.).max(0.) as usize,
            (ctx.camera_rect[1] / self.cell_height as f64 - 1.).max(0.) as usize
        ];
        let cull_limit = [
            1 + cull_start[0] + ctx.camera_rect[2] as usize / self.cell_width,
            1 + cull_start[1] + ctx.camera_rect[3] as usize / self.cell_height
        ];
        if self.draw_shadows {
            // Draws a few extra rows/cols to avoid shadows popping in
            let x_range = (cull_start[0])..(self.width.min(cull_limit[0] + 2));
            let y_range = (cull_start[1])..(self.height.min(cull_limit[1] + 5));
            for y in y_range.clone() {
                for x in x_range.clone() {
                    let idx = (y * self.width) + x;
                    let tile_i = self.tiles[idx];
                    if tile_i.1 {
                        self.pass(idx, x, y, tile_i.0, ctx, true);
                    }
                }
            }
        }
        let x_range = (cull_start[0])..(self.width.min(cull_limit[0] + 2));
        let y_range = (cull_start[1])..(self.height.min(cull_limit[1] + 2));
        for y in y_range {
            for x in x_range.clone() {
                let idx = (y * self.width) + x;
                let tile_i = self.tiles[idx];
                self.pass(idx, x, y, tile_i.0, ctx, false);
                z_order_render(ctx, game_ctx, x, y);
            }
        }
        perf().end("tilemap");
    }

    fn pass(&self, idx: usize, x: usize, y: usize, tile_i: usize, ctx: &mut RenderContext, shadow_pass: bool) {
        enum TextureType {
            Image(Arc<super::assets::Image>),
            ImageSheet(Arc<super::assets::ImageSheet>, usize)
        }

        let texture;
        let size;
        match &self.tileset.tiles[tile_i] {
            Tile::Empty => {
                return;
            },
            Tile::SingleTile(tile) => {
                let image = assets().image(&tile.image);
                size = [image.size.x() as f64, image.size.y() as f64];
                texture = TextureType::Image(image);
            },
            Tile::TileRandom(tile) => {
                let sheet = assets().image_sheet(&tile.image_sheet.path, tile.image_sheet.tile_size.clone());
                size = [sheet.tile_size.x() as f64, sheet.tile_size.y() as f64];
                let mut rng = Rng::new(idx as u32);
                let i = rng.randu_range(0, sheet.len());
                texture = TextureType::ImageSheet(sheet, i)
            },
            Tile::T16Subset(tile) => {
                let sheet = assets().image_sheet(&tile.image_sheet.path, tile.image_sheet.tile_size.clone());
                size = [sheet.tile_size.x() as f64, sheet.tile_size.y() as f64];
                let mut u = true;
                if y > 0 {
                    u = self.tiles[idx - self.width].0 == tile_i;
                }
                let mut d = true;
                if y < self.height - 1 {
                    d = self.tiles[idx + self.width].0 == tile_i;
                }
                let mut l = true;
                if x > 0 {
                    l = self.tiles[idx - 1].0 == tile_i;
                }
                let mut r = true;
                if x < self.width - 1 {
                    r = self.tiles[idx + 1].0 == tile_i;
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
                texture = TextureType::ImageSheet(sheet, subtile_i);
            }
        }
        let pos = [
            x as f64 * self.cell_width as f64 - (size[0] - self.cell_width as f64) / 2.,
            y as f64 * self.cell_height as f64 - (size[1] - self.cell_height as f64),
        ];
        let texture = match &texture {
            TextureType::Image(img) => &img.texture,
            TextureType::ImageSheet(img, i) => &img.get(*i).unwrap()
        };
        if shadow_pass {
            let transform = ctx.context.transform.trans(pos[0], pos[1]).shear(-0.4, 0.).scale(1., 1.5).trans(size[0] * 0.4, -size[1] * 0.5);
            Image::new().color([0., 0.1, 0.5, 0.3]).draw(texture, &Default::default(), transform, ctx.gl);
        } else {
            let transform = ctx.context.transform.trans(pos[0], pos[1]);
            Image::new().draw(texture, &Default::default(), transform, ctx.gl);
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
    image: String
}

impl TileSingle {
    pub(crate) fn new(image: String) -> Self {
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