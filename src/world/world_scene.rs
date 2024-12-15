use ::image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};
use graphics::rectangle::{square, Border};
use piston::{Button, Key};

use crate::{engine::{geometry::Coord2, render::RenderContext, scene::{Scene, Update}, Color}, game::{actor::Actor, InputEvent}, literature::biography::BiographyWriter, world::species::SpeciesIntelligence};

use super::world::World;

pub struct WorldScene {
    pub world: World,
    pub player: Actor,
    pub cursor: Coord2
}

impl WorldScene {
    pub fn new(world: World, player: Actor) -> WorldScene {
        let cursor = Coord2::xy(world.map.size.x() as i32 / 2, world.map.size.y() as i32 / 2);
        WorldScene {
            player,
            world,
            cursor
        }
    }

}

impl Scene for WorldScene {
    fn render(&self, ctx: &mut RenderContext) {
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

        let spritesheet = ImageReader::open("./assets/sprites/settlements.png").unwrap().decode().unwrap();

        let settings = TextureSettings::new().filter(Filter::Nearest);
        let sett_textures = [
            Texture::from_image(&spritesheet.crop_imm(0*16, 0, 16, 16).to_rgba8(), &settings),
            Texture::from_image(&spritesheet.crop_imm(1*16, 0, 16, 16).to_rgba8(), &settings),
            Texture::from_image(&spritesheet.crop_imm(2*16, 0, 16, 16).to_rgba8(), &settings),
            Texture::from_image(&spritesheet.crop_imm(3*16, 0, 16, 16).to_rgba8(), &settings),
            Texture::from_image(&spritesheet.crop_imm(4*16, 0, 16, 16).to_rgba8(), &settings),
            Texture::from_image(&spritesheet.crop_imm(5*16, 0, 16, 16).to_rgba8(), &settings),
            Texture::from_image(&spritesheet.crop_imm(6*16, 0, 16, 16).to_rgba8(), &settings),
        ];
        
        let world = &self.world;

        let ts = 16.;

        let cursor_pos_on_screen = (900., 500.);

        let xoff = (-self.cursor.x as f64 * ts) + cursor_pos_on_screen.0;
        let yoff = (-self.cursor.y as f64 * ts) + cursor_pos_on_screen.1;

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
                rectangle(color.f32_arr(), rectangle::square(xoff + (x as f64 * ts), yoff + (y as f64 * ts), ts), ctx.context.transform, ctx.gl);

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
                    rectangle(black.alpha(opacity).f32_arr(), rectangle::square(xoff + (x as f64 * ts), yoff + (y as f64 * ts), ts), ctx.context.transform, ctx.gl);
                } else {
                    let opacity = height_diff;
                    rectangle(white.alpha(opacity).f32_arr(), rectangle::square(xoff + (x as f64 * ts), yoff + (y as f64 * ts), ts), ctx.context.transform, ctx.gl);
                }

            }   
        }

        let mut hover_settlement = None;

        for (id, settlement) in world.settlements.iter() {
            let settlement = settlement.borrow();

            if settlement.demographics.population > 0 {
                let color = faction_colors[settlement.faction_id.seq() % faction_colors.len()];
                let mut transparent = color.f32_arr();
                transparent[3] = 0.4;

                let mut rectangle = Rectangle::new(transparent);
                rectangle = rectangle.border(Border { color: color.f32_arr(), radius: 1.0 });
                let dims = square(xoff + (settlement.xy.0 as f64 * ts), yoff + (settlement.xy.1 as f64 * ts), ts);
                rectangle.draw(dims, &DrawState::default(), ctx.context.transform, ctx.gl);
            }
            let transform = ctx.context.transform.trans(xoff + (settlement.xy.0 as f64*ts), yoff + (settlement.xy.1 as f64*ts));

            let texture;
            if settlement.demographics.population == 0 {
                texture = &sett_textures[6];
            } else if settlement.demographics.population < 10 {
                texture = &sett_textures[0];
            } else if settlement.demographics.population < 25 {
                texture = &sett_textures[1];
            } else if settlement.demographics.population < 50 {
                texture = &sett_textures[2];
            } else if settlement.demographics.population < 100 {
                texture = &sett_textures[3];
            } else if settlement.demographics.population < 250 {
                texture = &sett_textures[4];
            } else {
                texture = &sett_textures[5];
            }

            image(texture, transform, ctx.gl);

            if settlement.xy.0 as i32 == self.cursor.x && settlement.xy.1 as i32 == self.cursor.y {
                hover_settlement = Some(id);
            }

        }

        // Render great beasts covens
        for (_, person) in world.people.iter() {
            let person = person.borrow();
            let species = world.species.get(&person.species).unwrap();

            if species.intelligence == SpeciesIntelligence::Instinctive {
                ctx.image("coven.png", [xoff + (person.position.x as f64 * ts), yoff + (person.position.y as f64 * ts)]);
            }
        }


        let mut color = white.f32_arr();
        color[3] = 0.7;
        rectangle(color, rectangle::square(cursor_pos_on_screen.0, cursor_pos_on_screen.1, ts), ctx.context.transform, ctx.gl);

        let tile = world.map.tile(self.cursor.x as usize, self.cursor.y as usize);
        let biography = BiographyWriter::new(&world);

        let mut text = biography.tile(&tile);

        if let Some(hover_settlement) = hover_settlement {
            text = format!("{}\n{}", text, biography.settlement(&hover_settlement));
        }
        let mut y = 16.0;
        for line in text.split('\n') {
            ctx.text(line, 10, [1040., y], white);
            y = y + 16.0;
        }
        
    }

    fn update(&mut self, _update: &Update) {
        
    }

    fn input(&mut self, evt: &InputEvent) {
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