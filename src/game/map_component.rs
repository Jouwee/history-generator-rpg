use crate::{engine::{assets::{assets, ImageSheetAsset}, geometry::{Coord2, Size2D}, gui::{button::Button, layout_component::LayoutComponent, UIEvent, UINode}, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, render::RenderContext, tilemap::{Tile16Subset, TileMap, TileSet, TileSingle}, COLOR_WHITE}, world::{topology::WorldTopology, site::{Site, SiteId, SiteType}, world::World}, GameContext};

pub(crate) struct MapComponent {
    layout: LayoutComponent,
    tilemap: LayeredDualgridTilemap,
    objects: TileMap,
    pub(crate) names: Vec<(Coord2, String, bool)>,
}

impl MapComponent {

    pub(crate) fn new() -> MapComponent {
        let mut dual_tileset = LayeredDualgridTileset::new();
        let image = ImageSheetAsset::new("map_tiles/grassland.png", Size2D(16, 16));
        dual_tileset.add(4, image);
        let image = ImageSheetAsset::new("map_tiles/forest.png", Size2D(16, 16));
        dual_tileset.add(5, image);

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
                self.tilemap.set_tile(x, y, tile.region_id as usize);
            }
        }
    }

    pub(crate) fn update_visible_sites<F>(&mut self, world: &World, predicate: F) where F: Fn(&SiteId, &Site) -> bool {
        self.names.clear();
        self.objects.reset();
        for site_id in world.sites.iter_ids::<SiteId>() {
            let site = world.sites.get(&site_id);
            if !predicate(&site_id, &site) {
                continue;
            }
            let mut major_name = false;
            let tile = match site.site_type {
                SiteType::Village => {
                    major_name = true;
                    if site.creatures.len() > 20 {
                        5
                    } else if site.creatures.len() > 5 {
                        1
                    } else if site.creatures.len() > 0 {
                        6
                    } else {
                        major_name = false;
                        4
                    }
                },
                SiteType::VarningrLair | SiteType::BanditCamp | SiteType::WolfPack => {
                    if site.creatures.len() > 0 {
                        3
                    } else {
                        continue;
                    }
                },
            };
            self.objects.set_tile(site.xy.x as usize, site.xy.y as usize, tile);
            self.names.push((site.xy, site.name().to_string(), major_name));
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