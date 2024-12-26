use std::time::{Duration, Instant};

use graphics::rectangle::{square, Border};
use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::{engine::{layered_dualgrid_tilemap::{LayeredDualgridTilemap, LayeredDualgridTileset}, render::RenderContext, scene::{Scene, Update}, Color}, game::InputEvent, world::species::SpeciesIntelligence};

use super::{history_generator::{WorldHistoryGenerator, WorldGenerationParameters}, world::World};

pub struct WorldGenScene {
    generator: WorldHistoryGenerator,
    total_time: Duration,
    tilemap: LayeredDualgridTilemap,
    banner_texture: Texture
}

impl WorldGenScene {
    pub fn new(params: WorldGenerationParameters) -> WorldGenScene {
        let spritesheet = ImageReader::open("assets/sprites/banner.png").unwrap().decode().unwrap();
        let settings = TextureSettings::new().filter(Filter::Nearest);

        let mut dual_tileset = LayeredDualgridTileset::new();
        let image = ImageReader::open("assets/sprites/world_tiles/ocean.png").unwrap().decode().unwrap();
        dual_tileset.add(1, image, 4, 4);
        let image = ImageReader::open("assets/sprites/world_tiles/coast.png").unwrap().decode().unwrap();
        dual_tileset.add(0, image, 4, 4);
        let image = ImageReader::open("assets/sprites/world_tiles/grassland.png").unwrap().decode().unwrap();
        dual_tileset.add(4, image, 4, 4);
        let image = ImageReader::open("assets/sprites/world_tiles/forest.png").unwrap().decode().unwrap();
        dual_tileset.add(5, image, 4, 4);
        let image = ImageReader::open("assets/sprites/world_tiles/desert.png").unwrap().decode().unwrap();
        dual_tileset.add(3, image, 4, 4);

        let mut scene = WorldGenScene {
            generator: WorldHistoryGenerator::seed_world(params),
            total_time: Duration::new(0, 0),
            tilemap: LayeredDualgridTilemap::new(dual_tileset, 256, 256, 4, 4),
            banner_texture: Texture::from_image(&spritesheet.to_rgba8(), &settings)
        };
        scene.build_tilemap();
        return scene
    }

    pub fn build_tilemap(&mut self) {
        let map = &self.generator.world.map;
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

    pub fn into_world(self) -> World {
        return self.generator.world
    }
}

impl Scene for WorldGenScene {
    fn render(&mut self, ctx: &mut RenderContext) {
        use graphics::*;

        // https://lospec.com/palette-list/31
        let gray = Color::from_hex("636663");
        // let XXX = Color::from_hex("87857c");
        // let XXX = Color::from_hex("bcad9f");
        // let salmon = Color::from_hex("f2b888");
        let orange = Color::from_hex("eb9661");
        let red = Color::from_hex("b55945");
        // let XXX = Color::from_hex("734c44");
        // let XXX = Color::from_hex("3d3333");
        let wine = Color::from_hex("593e47");
        // let XXX = Color::from_hex("7a5859");
        // let XXX: Color = Color::from_hex("a57855");
        let yellow = Color::from_hex("de9f47");
        // let XXX = Color::from_hex("fdd179");
        // let off_white = Color::from_hex("fee1b8");
        // let XXX = Color::from_hex("d4c692");
        // let XXX = Color::from_hex("a6b04f");
        let yellow_green = Color::from_hex("819447");
        // let XXX = Color::from_hex("44702d");
        // let dark_green = Color::from_hex("2f4d2f");
        // let XXX = Color::from_hex("546756");
        // let XXX = Color::from_hex("89a477");
        // let XXX = Color::from_hex("a4c5af");
        let teal = Color::from_hex("cae6d9");
        let white = Color::from_hex("f1f6f0");
        // let XXX = Color::from_hex("d5d6db");
        // let XXX = Color::from_hex("bbc3d0");
        // let XXX = Color::from_hex("96a9c1");
        // let XXX = Color::from_hex("6c81a1");
        let blue = Color::from_hex("405273");
        // let XXX = Color::from_hex("303843");
        let black = Color::from_hex("14233a");

        let faction_colors = [red, black, blue, teal, yellow, yellow_green, wine, white, orange, gray];

        let world = &self.generator.world;

        let ts = 4.;

        self.tilemap.render(ctx);

        for (_, settlement) in world.settlements.iter() {
            let settlement = settlement.borrow();

            if settlement.demographics.population > 0 {
                let color = faction_colors[settlement.faction_id.seq() % faction_colors.len()];
                let mut transparent = color.f32_arr();
                transparent[3] = 0.4;

                let mut rectangle = Rectangle::new(transparent);
                rectangle = rectangle.border(Border { color: color.f32_arr(), radius: 1.0 });
                let dims = square(settlement.xy.0 as f64 * ts, settlement.xy.1 as f64 * ts, ts);
                rectangle.draw(dims, &DrawState::default(), ctx.context.transform, ctx.gl);
            }

        }

        // Render great beasts
        for (_, person) in world.people.iter() {
            let person = person.borrow();
            let species = world.species.get(&person.species).unwrap();

            if species.intelligence == SpeciesIntelligence::Instinctive {
                ctx.circle([person.position.x as f64 * ts, person.position.y as f64 * ts, ts, ts], red);
            }
        }

        // Year banner
        let center = ctx.layout_rect[2] / 2.;
        ctx.texture_ref(&self.banner_texture, [center - 64., 0.]);
        let text = format!("Year {}", &self.generator.year.to_string());
        let text_width = ctx.default_font.width(11, &text).unwrap_or(0.);
        ctx.text(&text, 11, [(center - text_width / 2.).round(), 16.], white);
        let text = "Press <enter> to start playing";
        let text_width = ctx.default_font.width(11, &text).unwrap_or(0.);
        ctx.text(&text, 11, [(center - text_width / 2.).round(), 40.], white);
    }

    fn update(&mut self, _update: &Update) {
        let start = Instant::now();
        loop {
            if self.generator.year < 750 {
                println!("Year {}, {} people to process", self.generator.year, self.generator.world.people.len());
                let now = Instant::now();
                self.generator.simulate_year();
                self.total_time = self.total_time + now.elapsed();
                if self.generator.year % 10 == 0 {
                    println!("Elapsed: {:.2?}", self.total_time)
                }
            }
            // Simulate years until reach the max time per iteration, otherwise it takes longer than it needs
            if start.elapsed().as_secs_f64() >= _update.max_update_time {
                break;
            }
        }
    }

    fn input(&mut self, _evt: &InputEvent) {
    }
}