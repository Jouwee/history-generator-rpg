use ::image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};
use piston::{Button, Key};

use crate::{commons::history_vec::Id, engine::{audio::TrackMood, geometry::Coord2, layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, render::RenderContext, scene::{Scene, Update}, Color}, game::{actor::Actor, codex::knowledge_codex::KnowledgeCodex, InputEvent}, GameContext};

use super::world::World;

pub struct WorldScene {
    pub world: World,
    pub player: Actor,
    pub codex: KnowledgeCodex,
    tilemap: LayeredDualgridTilemap,
    banner_texture: Texture,
    settlement_textures: [Texture; 7],
    pub cursor: Coord2
}

impl WorldScene {
    pub fn new(world: World, player: Actor, codex: KnowledgeCodex) -> WorldScene {
        let cursor = Coord2::xy(world.map.size.x() as i32 / 2, world.map.size.y() as i32 / 2);
        let banner = ImageReader::open("assets/sprites/banner.png").unwrap().decode().unwrap();
        let settings = TextureSettings::new().filter(Filter::Nearest);


        let mut dual_tileset = LayeredDualgridTileset::new();
        let image = ImageReader::open("assets/sprites/world_tiles_large/ocean.png").unwrap().decode().unwrap();
        dual_tileset.add(1, image, 16, 16);
        let image = ImageReader::open("assets/sprites/world_tiles_large/coast.png").unwrap().decode().unwrap();
        dual_tileset.add(0, image, 16, 16);
        let image = ImageReader::open("assets/sprites/world_tiles_large/grassland.png").unwrap().decode().unwrap();
        dual_tileset.add(4, image, 16, 16);
        let image = ImageReader::open("assets/sprites/world_tiles_large/forest.png").unwrap().decode().unwrap();
        dual_tileset.add(5, image, 16, 16);
        let image = ImageReader::open("assets/sprites/world_tiles_large/desert.png").unwrap().decode().unwrap();
        dual_tileset.add(3, image, 16, 16);

        let spritesheet = ImageReader::open("./assets/sprites/settlements.png").unwrap().decode().unwrap();

        let mut scene = WorldScene {
            player,
            codex,
            world,
            cursor,
            tilemap: LayeredDualgridTilemap::new(dual_tileset, 256, 256, 16, 16),
            banner_texture: Texture::from_image(&banner.to_rgba8(), &settings),
            settlement_textures: [
                Texture::from_image(&spritesheet.crop_imm(0*16, 0, 16, 16).to_rgba8(), &settings),
                Texture::from_image(&spritesheet.crop_imm(1*16, 0, 16, 16).to_rgba8(), &settings),
                Texture::from_image(&spritesheet.crop_imm(2*16, 0, 16, 16).to_rgba8(), &settings),
                Texture::from_image(&spritesheet.crop_imm(3*16, 0, 16, 16).to_rgba8(), &settings),
                Texture::from_image(&spritesheet.crop_imm(4*16, 0, 16, 16).to_rgba8(), &settings),
                Texture::from_image(&spritesheet.crop_imm(5*16, 0, 16, 16).to_rgba8(), &settings),
                Texture::from_image(&spritesheet.crop_imm(6*16, 0, 16, 16).to_rgba8(), &settings),
            ]
        };
        scene.build_tilemap();
        return scene
    }

    fn build_tilemap(&mut self) {
        let map = &self.world.map;
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


    fn location_name(&self, cursor: Coord2, hover_settlement: Option<Id>) -> String {
        if let Some(id) = hover_settlement {
            let settlement = self.world.settlements.get(&id);
            return settlement.name.clone()
        } else {
            let tile = self.world.map.tile(cursor.x as usize, cursor.y as usize);
            let region = self.world.regions.get(&Id(tile.region_id as usize)).unwrap();
            return region.name.clone()
        }
    }

}

impl Scene for WorldScene {
    fn init(&mut self, ctx: &mut GameContext) {
        ctx.audio.switch_music(TrackMood::Regular);
    }

    fn render(&mut self, ctx: &mut RenderContext, _game_ctx: &GameContext) {
        ctx.pixel_art(2);
        use graphics::*;

        let white = Color::from_hex("f1f6f0");
        
        let world = &self.world;

        let ts = 16.;

        ctx.push();
        let cursor = [self.cursor.x as f64 * ts, self.cursor.y as f64 * ts];
        ctx.center_camera_on(cursor);

        self.tilemap.render(ctx);

        let mut hover_settlement = None;
        for (id, settlement) in world.settlements.iter() {
            let settlement = settlement.borrow();
            let transform = ctx.context.transform.trans(settlement.xy.0 as f64*ts, settlement.xy.1 as f64*ts);
            let texture;
            if settlement.demographics.population == 0 {
                texture = &self.settlement_textures[6];
            } else if settlement.demographics.population < 10 {
                texture = &self.settlement_textures[0];
            } else if settlement.demographics.population < 25 {
                texture = &self.settlement_textures[1];
            } else if settlement.demographics.population < 50 {
                texture = &self.settlement_textures[2];
            } else if settlement.demographics.population < 100 {
                texture = &self.settlement_textures[3];
            } else if settlement.demographics.population < 250 {
                texture = &self.settlement_textures[4];
            } else {
                texture = &self.settlement_textures[5];
            }

            image(texture, transform, ctx.gl);

            if settlement.xy.0 as i32 == self.cursor.x && settlement.xy.1 as i32 == self.cursor.y {
                hover_settlement = Some(id);
            }

        }

        ctx.image("cursor.png", cursor);
        let _ = ctx.try_pop();

        // Location banner
        let center = ctx.layout_rect[2] / 2.;
        ctx.texture_ref(&self.banner_texture, [center - 64., 0.]);
        let text = self.location_name(self.cursor, hover_settlement);
        let text_width = ctx.default_font.width(11, &text).unwrap_or(0.);
        ctx.text(&text, 11, [(center - text_width / 2.).round(), 16.], white);
        let text = "Press <enter> to enter";
        let text_width = ctx.default_font.width(11, &text).unwrap_or(0.);
        ctx.text(&text, 11, [(center - text_width / 2.).round(), 40.], white);
    }

    fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {
        
    }

    fn input(&mut self, evt: &InputEvent, _ctx: &mut GameContext) {
        match evt.button_args.button {
            Button::Keyboard(Key::Up) => {
                if self.cursor.y > 0 {
                    self.cursor.y -= 1;
                }
            },
            Button::Keyboard(Key::Down) => {
                if self.cursor.y < self.world.map.size.y() as i32 - 1 {
                    self.cursor.y += 1;
                }
            },
            Button::Keyboard(Key::Left) => {
                if self.cursor.x > 0 {
                    self.cursor.x -= 1;
                }
            },
            Button::Keyboard(Key::Right) => {
                if self.cursor.x < self.world.map.size.x() as i32 - 1 {
                    self.cursor.x += 1;
                }
            }
            _ => (),
        }
    }

}