use std::time::{Duration, Instant};

use graphics::rectangle::{square, Border};
use piston::{Button, Key};

use crate::{engine::{render::RenderContext, scene::{Scene, Update}, Color}, game::InputEvent, world::species::SpeciesIntelligence};

use super::{history_generator::{WorldHistoryGenerator, WorldGenerationParameters}, world::World};

pub struct WorldGenScene {
    generator: WorldHistoryGenerator,
    view: WorldViewMode,
    total_time: Duration
}

impl WorldGenScene {
    pub fn new(params: WorldGenerationParameters) -> WorldGenScene {
        WorldGenScene {
            generator: WorldHistoryGenerator::seed_world(params),
            total_time: Duration::new(0, 0),
            view: WorldViewMode::Normal
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
        let salmon = Color::from_hex("f2b888");
        let orange = Color::from_hex("eb9661");
        let red = Color::from_hex("b55945");
        // let XXX = Color::from_hex("734c44");
        // let XXX = Color::from_hex("3d3333");
        let wine = Color::from_hex("593e47");
        // let XXX = Color::from_hex("7a5859");
        // let XXX: Color = Color::from_hex("a57855");
        let yellow = Color::from_hex("de9f47");
        // let XXX = Color::from_hex("fdd179");
        let off_white = Color::from_hex("fee1b8");
        // let XXX = Color::from_hex("d4c692");
        // let XXX = Color::from_hex("a6b04f");
        let yellow_green = Color::from_hex("819447");
        // let XXX = Color::from_hex("44702d");
        let dark_green = Color::from_hex("2f4d2f");
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
        if self.view == WorldViewMode::Normal {


            for x in 0..world.map.size.x() {
                for y in 0..world.map.size.y() {
                    let tile = world.map.tile(x, y);

                    let color;
                    match tile.region_id {
                        0 => color = blue,
                        1 => color = off_white,
                        2 => color = dark_green,
                        3 => color = salmon,
                        _ => color = black
                    }
                    rectangle(color.f32_arr(), rectangle::square(x as f64 * ts, y as f64 * ts, ts), ctx.context.transform, ctx.gl);

                    let mut height_diff = 0.0;
                    let mut height_count = 0;
                    if x > 0 {
                        height_diff += tile.elevation as f32 - world.map.tile(x - 1, y).elevation as f32;
                        height_count += 1;
                    }
                    if y > 0 {
                        height_diff += tile.elevation as f32 - world.map.tile(x, y - 1).elevation as f32;
                        height_count += 1;
                    }
                    if x < world.map.size.x() - 1 {
                        height_diff += world.map.tile(x + 1, y).elevation as f32 - tile.elevation as f32;
                        height_count += 1;
                    }
                    if y < world.map.size.y() - 1 {
                        height_diff += world.map.tile(x, y + 1).elevation as f32 - tile.elevation as f32;
                        height_count += 1;
                    }
                    height_diff = (height_diff / height_count as f32) / 256.0;
                    if height_diff < 0.0 {
                        let opacity = height_diff.abs();
                        rectangle(black.alpha(opacity).f32_arr(), rectangle::square(x as f64 * ts, y as f64 * ts, ts), ctx.context.transform, ctx.gl);
                    } else {
                        let opacity = height_diff;
                        rectangle(white.alpha(opacity).f32_arr(), rectangle::square(x as f64 * ts, y as f64 * ts, ts), ctx.context.transform, ctx.gl);
                    }

                }   
            }

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

        } else {
            for x in 0..world.map.size.x() {
                for y in 0..world.map.size.y() {
                    let tile = world.map.tile(x, y);
                    let mut color = white;
                    match self.view {
                        WorldViewMode::Normal => (), // Already checked
                        WorldViewMode::Elevation => {
                            color = white.alpha((tile.elevation as f32) / 256.0);
                        },
                        WorldViewMode::Precipitation => {
                            color = blue.alpha((tile.precipitation as f32) / 256.0);
                        }
                    }
                    rectangle(color.f32_arr(), rectangle::square(x as f64 * ts, y as f64 * ts, ts), ctx.context.transform, ctx.gl);
                }   
            }
        }

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

    fn input(&mut self, evt: &InputEvent) {
        match evt.button_args.button {
            Button::Keyboard(Key::V) => {
                match self.view {
                    WorldViewMode::Normal => self.view = WorldViewMode::Elevation,
                    WorldViewMode::Elevation => self.view = WorldViewMode::Precipitation,
                    WorldViewMode::Precipitation => self.view = WorldViewMode::Normal,
                }
            }
            _ => ()
        }
    }
}

#[derive(PartialEq)]
enum WorldViewMode {
    Normal,
    Elevation,
    Precipitation,
}