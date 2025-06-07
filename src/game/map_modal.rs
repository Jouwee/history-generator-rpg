use crate::{engine::{asset::{image::ImageAsset, image_sheet::ImageSheetAsset}, geometry::{Coord2, Size2D, Vec2}, gui::{button::Button, UINode}, input::InputEvent, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, render::RenderContext, scene::Update, tilemap::{Tile16Subset, TileMap, TileSet, TileSingle}, Color}, world::{unit::UnitType, world::World}, GameContext};
use piston::{Button as Btn, ButtonState, Key, MouseButton};

use super::InputEvent as OldInputEvent;

pub(crate) struct MapModal {
    world_size: Size2D,
    tilemap: LayeredDualgridTilemap,
    objects: TileMap,
    offset: Vec2,
    player_pos: Coord2,
    close_button: Button
}

impl MapModal {

    pub(crate) fn new() -> MapModal {
        let mut dual_tileset = LayeredDualgridTileset::new();
        let image = ImageSheetAsset::new("map_tiles/ocean.png", Size2D(16, 16));
        dual_tileset.add(1, image);
        let image = ImageSheetAsset::new("map_tiles/coast.png", Size2D(16, 16));
        dual_tileset.add(0, image);
        let image = ImageSheetAsset::new("map_tiles/grassland.png", Size2D(16, 16));
        dual_tileset.add(4, image);
        let image = ImageSheetAsset::new("map_tiles/forest.png", Size2D(16, 16));
        dual_tileset.add(5, image);
        let image = ImageSheetAsset::new("map_tiles/desert.png", Size2D(16, 16));
        dual_tileset.add(3, image);

        let mut tileset = TileSet::new();
        let image = ImageAsset::new("map_tiles/settlement.png");
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = ImageSheetAsset::new("map_tiles/road.png", Size2D(16, 16));
        tileset.add(crate::engine::tilemap::Tile::T16Subset(Tile16Subset::new(image)));
        let image = ImageAsset::new("map_tiles/marker.png");
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));

        let mut close_button = Button::text("Close");
        close_button.layout_component().anchor_top_right(0., 0.);

        MapModal {
            tilemap: LayeredDualgridTilemap::new(dual_tileset, 256, 256, 16, 16),
            objects: TileMap::new(tileset, 256, 256, 16, 16),
            offset: Vec2::xy(128.*16., 128.*16.),
            player_pos: Coord2::xy(0, 0),
            world_size: Size2D(0, 0),
            close_button
        }
    }

    pub(crate) fn init(&mut self, world: &World, player_pos: &Coord2) {
        let map = &world.map;
        for x in 0..map.size.x() {
            for y in 0..map.size.y() {
                let tile = map.tile(x, y);
                match tile.region_id {
                    0 => self.tilemap.set_tile(x, y, 0),
                    1 => self.tilemap.set_tile(x, y, 1),
                    2 => self.tilemap.set_tile(x, y, 2),
                    3 => self.tilemap.set_tile(x, y, 3),
                    4 => self.tilemap.set_tile(x, y, 4),
                    _ => ()
                }
            }
        }

        for unit in world.units.iter() {
            let unit = unit.borrow();
            let tile = match unit.unit_type {
                UnitType::Village => 1,
                UnitType::BanditCamp => {
                    if unit.creatures.len() > 0 {
                        3
                    } else {
                        0
                    }
                },
            };
            self.objects.set_tile(unit.xy.x as usize, unit.xy.y as usize, tile);
            // Set grass as BG
            self.tilemap.set_tile(unit.xy.x as usize, unit.xy.y as usize, 2);
        }

        self.offset = Vec2::xy(player_pos.x as f32 * 16., player_pos.y as f32 * 16.);
        self.player_pos = player_pos.clone();

        self.world_size = world.map.size;

    }

    pub(crate) fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        ctx.push();
        ctx.center_camera_on([self.offset.x as f64, self.offset.y as f64]);
        self.tilemap.render(ctx, game_ctx);
        self.objects.render(ctx, game_ctx, |_, _, _, _| {});

        let cursor = [self.player_pos.x * 16, self.player_pos.y * 16];
        
        let cursor_clamp = [
            cursor[0].clamp(ctx.camera_rect[0] as i32, ctx.camera_rect[0] as i32 + ctx.camera_rect[2] as i32 - 16),
            cursor[1].clamp(ctx.camera_rect[1] as i32, ctx.camera_rect[1] as i32 + ctx.camera_rect[3] as i32 - 16),
        ];
        if cursor != cursor_clamp {
            ctx.image(&ImageAsset::new("map_tiles/player_offscreen.png"), cursor_clamp, &mut game_ctx.assets);
        } else {
            ctx.image(&ImageAsset::new("map_tiles/player.png"), cursor_clamp, &mut game_ctx.assets);
        }
        let _ = ctx.try_pop();
        // Control
        ctx.image(&ImageAsset::new("controls/right_click.png"), [ctx.layout_rect[2] as i32 - 88, ctx.layout_rect[3] as i32 - 24], &mut game_ctx.assets);
        ctx.text("Drag to move", game_ctx.assets.font_standard(), [ctx.layout_rect[2] as i32 - 72, ctx.layout_rect[3] as i32 - 14], &Color::from_hex("ffffff"));
        self.close_button.render(&(), ctx, game_ctx);
    }

    pub(crate) fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {}

    pub(crate) fn input(&mut self, evt: &OldInputEvent, ctx: &mut GameContext) -> MapModalEvent {
        if evt.button_args.state == ButtonState::Press {
            match evt.button_args.button {
                Btn::Keyboard(Key::M) | Btn::Keyboard(Key::Escape) => {
                    return MapModalEvent::Close
                }
                _ => ()
            }
        }
        if self.close_button.input(&mut (), &evt.evt, ctx).is_break() {
            return MapModalEvent::Close;
        }
        let camera = ctx.display_context.camera_rect;
        let clamp = [
            [camera[2] / 2. + 16., (self.world_size.0 as f64 * 16.) - camera[2] / 2.],
            [camera[3] / 2. + 16., (self.world_size.1 as f64 * 16.) - camera[3] / 2.],
        ];
        match evt.evt {
            InputEvent::Drag { button: MouseButton::Left, offset } => {
                self.offset.x = (self.offset.x - offset[0] as f32).clamp(clamp[0][0] as f32, clamp[0][1] as f32);
                self.offset.y = (self.offset.y - offset[1] as f32).clamp(clamp[1][0] as f32, clamp[1][1] as f32);
            },
            InputEvent::Click { button: MouseButton::Left, pos } => {
                let coord = Coord2::xy(
                    ((pos[0] + camera[0] as f64) / 16.) as i32, 
                    ((pos[1] + camera[1] as f64) / 16.) as i32
                );
                return MapModalEvent::InstaTravelTo(coord)
            }
            _ => ()
        }
        return MapModalEvent::None
    }

}

pub(crate) enum MapModalEvent {
    None,
    Close,
    InstaTravelTo(Coord2)
}