use crate::{engine::{assets::{assets, ImageSheetAsset}, geometry::{Coord2, Size2D}, gui::{button::Button, layout_component::LayoutComponent, UIEvent, UINode}, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, render::RenderContext, tilemap::{Tile16Subset, TileMap, TileSet, TileSingle}, COLOR_WHITE}, world::{topology::WorldTopology, unit::{Unit, UnitId, UnitType}, world::World}, GameContext};

pub(crate) struct MapComponent {
    layout: LayoutComponent,
    tilemap: LayeredDualgridTilemap,
    objects: TileMap,
    pub(crate) names: Vec<(Coord2, String, bool)>,
}

impl MapComponent {

    pub(crate) fn new() -> MapComponent {
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
        let image = String::from("map_tiles/settlement.png");
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = ImageSheetAsset::new("map_tiles/road.png", Size2D(16, 16));
        tileset.add(crate::engine::tilemap::Tile::T16Subset(Tile16Subset::new(image)));
        let image = String::from("map_tiles/marker.png");
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = String::from("map_tiles/settlement_ruins.png");
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = String::from("map_tiles/settlement_big.png");
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));
        let image = String::from("map_tiles/settlement_small.png");
        tileset.add(crate::engine::tilemap::Tile::SingleTile(TileSingle::new(image)));

        let mut close_button = Button::text("Close");
        close_button.layout_component().anchor_top_right(0., 0.);

        MapComponent {
            layout: LayoutComponent::new(),
            tilemap: LayeredDualgridTilemap::new(dual_tileset, 256, 256, 16, 16),
            objects: TileMap::new(tileset, 256, 256, 16, 16),
            names: Vec::new(),
        }
    }

    pub(crate) fn set_topology(&mut self, map: &WorldTopology) {
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
    }

    pub(crate) fn update_visible_units<F>(&mut self, world: &World, predicate: F) where F: Fn(&UnitId, &Unit) -> bool {
        self.names.clear();
        self.objects.reset();
        for unit_id in world.units.iter_ids::<UnitId>() {
            let unit = world.units.get(&unit_id);
            if !predicate(&unit_id, &unit) {
                continue;
            }
            let mut major_name = false;
            let tile = match unit.unit_type {
                UnitType::Village => {
                    major_name = true;
                    if unit.creatures.len() > 20 {
                        5
                    } else if unit.creatures.len() > 5 {
                        1
                    } else if unit.creatures.len() > 0 {
                        6
                    } else {
                        major_name = false;
                        4
                    }
                },
                UnitType::VarningrLair | UnitType::BanditCamp | UnitType::WolfPack => {
                    if unit.creatures.len() > 0 {
                        3
                    } else {
                        continue;
                    }
                },
            };
            self.objects.set_tile(unit.xy.x as usize, unit.xy.y as usize, tile);
            self.names.push((unit.xy, unit.name().to_string(), major_name));
        }
    }

}

impl UINode for MapComponent {
    type State = ();
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        self.tilemap.render(ctx);
        self.objects.render(ctx, game_ctx, |_, _, _, _| {});

        let mut assets = assets();
        let font = assets.font_heading();
        for (coord, name, major_name) in self.names.iter() {
            if *major_name {
                let width = font.width(name);
                ctx.text_shadow(name, font, [coord.x * 16 - (width / 2.) as i32 + 8, coord.y * 16 + 24], &COLOR_WHITE);
            }
        }
    }
}