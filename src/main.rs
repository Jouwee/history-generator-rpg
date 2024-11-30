extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;


use std::{cell::{Ref, RefCell, RefMut}, cmp::Ordering, collections::{BTreeMap, HashMap}, vec};
use commons::{history_vec::{HistoryVec, Id}, markovchains::MarkovChainSingleWordModel, rng::Rng, strings::Strings};
use engine::{Color, Point2D};
use literature::biography::BiographyWriter;
use ::image::ImageReader;
use noise::{NoiseFn, Perlin};
use graphics::rectangle::{square, Border};
use world::{event::*, faction::{Faction, FactionRelation}, settlement::{Settlement, SettlementBuilder}};

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, Texture, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::input::{Button, ButtonState, Key};
use piston::ButtonEvent;
use piston::window::WindowSettings;

pub mod engine;
pub mod commons;
pub mod literature;
pub mod world;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
}

impl App {
    fn render(&mut self, args: &RenderArgs, world: &WorldGraph, cursor: &Point2D) {
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
        ];
        
        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        let ref mut glyphs = GlyphCache::new("assets/Minecraft.ttf", (), texture_settings).expect("Could not load font");

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(black.f32_arr(), gl);

            for x in 0..WORLD_MAP_WIDTH {
                for y in 0..WORLD_MAP_WIDTH {
                    let tile = world.map.get_world_tile(x, y);
                    let color;
                    match tile.region_id {
                        0 => color = off_white,
                        1 => color = dark_green,
                        2 => color = salmon,
                        _ => color = black
                    }
                    rectangle(color.f32_arr(), rectangle::square(x as f64 * 16.0, y as f64 * 16.0, 16.0), c.transform, gl);
                }   
            }

            let mut hover_settlement = None;

            for (id, settlement) in world.settlements.iter() {
                let settlement = settlement.borrow();


                let color = faction_colors[settlement.faction_id.seq() % faction_colors.len()];
                let mut transparent = color.f32_arr();
                transparent[3] = 0.4;

                let mut rectangle = Rectangle::new(transparent);
                rectangle = rectangle.border(Border { color: color.f32_arr(), radius: 1.0 });
                let dims = square(settlement.xy.0 as f64 * 16.0, settlement.xy.1 as f64 * 16.0, 16.0);
                rectangle.draw(dims, &DrawState::default(), c.transform, gl);

                let transform = c.transform.trans(settlement.xy.0 as f64*16.0, settlement.xy.1 as f64*16.0);

                let texture;
                if settlement.demographics.population < 10 {
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

                image(texture, transform, gl);

                if settlement.xy == *cursor {
                    hover_settlement = Some(id);
                }

            }

            
            let mut color = white.f32_arr();
            color[3] = 0.7;
            rectangle(color, rectangle::square(cursor.0 as f64 * 16.0, cursor.1 as f64 * 16.0, 16.0), c.transform, gl);

            let tile = world.map.get_world_tile(cursor.0, cursor.1);
            let biography = BiographyWriter::new(&world);

            let mut text = biography.tile(&tile);

            if let Some(hover_settlement) = hover_settlement {
                text = format!("{}\n{}", text, biography.settlement(&hover_settlement));
            }
            let mut y = 16.0;
            for line in text.split('\n') {
                text::Text::new_color(white.f32_arr(), 10)
                    .draw(
                        line,
                        glyphs,
                        &c.draw_state,
                        c.transform.trans((WORLD_MAP_WIDTH * 16) as f64 + 16.0, y),
                        gl,
                    )
                    .unwrap();
                y = y + 16.0;
            }

            

        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
    }
}

#[derive(Clone)]
struct CulturePrefab {
    id: Id,
    name: String,
    language: LanguagePrefab,
    first_name_male_model: MarkovChainSingleWordModel,
    first_name_female_model: MarkovChainSingleWordModel,
    last_name_model: MarkovChainSingleWordModel,
}

#[derive(Clone)]
struct LanguagePrefab {
    dictionary: HashMap<String, String>
}


#[derive(Debug)]
struct RegionPrefab {
    name: String,
    id: usize,
    elevation: (u8, u8),
    temperature: (u8, u8),
    soil_fertility_range: (f32, f32),
    gold_generation_range: (f32, f32),
    fauna: Vec<String>,
    flora: Vec<String>,
}

struct WorldGraph {
    map: WorldMap,
    cultures: HashMap<Id, CulturePrefab>,
    factions: HistoryVec<Faction>,
    settlements: HistoryVec<Settlement>,
    people: People,
    events: WorldEvents
}

struct People {
    inner: BTreeMap<Id, RefCell<Person>>
}

impl People {
    
    fn new() -> People {
        People {
            inner: BTreeMap::new()
        }
    }

    fn get(&self, id: &Id) -> Option<Ref<Person>> {
        let option = self.inner.get(id);
        match option {
            None => None,
            Some(ref_cell) => Some(ref_cell.borrow())
        }
    }

    fn get_mut(&self, id: &Id) -> Option<RefMut<Person>> {
        let option = self.inner.get(id);
        match option {
            None => None,
            Some(ref_cell) => Some(ref_cell.borrow_mut())
        }
    }

    fn insert(&mut self, person: Person) {
        self.inner.insert(person.id, RefCell::new(person));
    }

    fn iter(&self) -> impl Iterator<Item = (&Id, &RefCell<Person>)> {
        return self.inner.iter().filter(|(_id, person)| person.borrow().simulatable())
    }

}

const WORLD_MAP_HEIGHT: usize = 64;
const WORLD_MAP_WIDTH: usize = 64;

struct WorldMap {
    elevation: [u8; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
    temperature: [u8; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
    soil_ferility: [f32; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
    region_id: [u8; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH]
}

impl WorldMap {

    pub fn get_world_tile(&self, x: usize, y: usize) -> WorldTileData {
        let i = (y * WORLD_MAP_WIDTH) + x;
        return WorldTileData {
            xy: Point2D(x, y),
            elevation: self.elevation[i],
            temperature: self.temperature[i],
            soil_fertility: self.soil_ferility[i],
            region_id: self.region_id[i],
        }
    }

}

#[derive(Debug)]
struct WorldTileData {
    xy: Point2D,
    elevation: u8,
    temperature: u8,
    soil_fertility: f32,
    region_id: u8
}

#[derive(Clone, PartialEq, Debug)]
enum PersonSex {
    Male,
    Female
}

impl PersonSex {

    fn opposite(&self) -> PersonSex {
        match self {
            PersonSex::Male => return PersonSex::Female,
            PersonSex::Female => return PersonSex::Male,
        }
    }

}

#[derive(Clone, Debug)]
struct Person {
    id: Id,
    first_name: String,
    last_name: String,
    birth_last_name: String,
    importance: Importance,
    birth: u32,
    sex: PersonSex,
    death: u32,
    culture_id: Id,
    next_of_kin: Vec<NextOfKin>,
    faction_id: Id,
    faction_relation: FactionRelation,
    leader_of_settlement: Option<Id>
}

#[derive(Clone, PartialEq, Debug)]
enum Importance {
    Important,
    Unimportant,
    Unknown
}

impl Importance {
    fn lower(&self) -> Importance {
        match self {
            Importance::Important => return Importance::Unimportant,
            Importance::Unimportant => return Importance::Unknown,
            Importance::Unknown => return Importance::Unknown,
        }
    }
}

impl Person {

    fn birth_name(&self) -> String {
        return format!("{} {}", self.first_name, self.birth_last_name)
    }

    fn name(&self) -> String {
        let title = "Commoner";
        return format!("{} {} ({:?}, {})", self.first_name, self.last_name, self.importance, title)
    }

    fn simulatable(&self) -> bool {
        self.alive() && self.importance != Importance::Unknown
    }

    fn alive(&self) -> bool {
        return self.death == 0
    }

    fn spouse(&self) -> Option<&Id> {
        let spouse = self.next_of_kin.iter().find(|r| r.relative == Relative::Spouse);
        if let Some(spouse) = spouse {
            return Some(&spouse.person_id)
        };
        return None
    }

    fn fertility(&self, year: u32) -> f32 {
        let age = (year - self.birth) as f32;
        // https://thefertilityshop.co.uk/wp-content/uploads/2021/12/bfs-monthly-fertility-by-age-1024x569.png
        if self.sex == PersonSex::Male {
            return f32::max(0.0, -(age / 60.0).powf(2.0) + 1.0)
        } else {
            return f32::max(0.0, -(age / 40.0).powf(6.0) + 1.0)
        }
    }

    fn find_next_of_kin(&self, relative: Relative) -> Option<&Id> {
        let spouse = self.next_of_kin.iter().find(|r| r.relative == relative);
        if let Some(spouse) = spouse {
            return Some(&spouse.person_id)
        };
        return None
    }

    fn sorted_heirs(&self) -> Vec<NextOfKin> {
        let priorities = [
            Relative::Child,
            Relative::Spouse,
            Relative::Sibling,
        ];
        
        let mut sorted = self.next_of_kin.clone();
        sorted.sort_by(|kin1, kin2| {
            let priority_1 = priorities.iter().position(|r| *r == kin1.relative);
            let priority_2 = priorities.iter().position(|r| *r == kin2.relative);
            if priority_1 != priority_2 {
                return priority_1.cmp(&priority_2);
            }
            return Ordering::Equal;
        });
        return sorted
    }

}

#[derive(Clone, Debug)]
struct NextOfKin {
    person_id: Id,
    relative: Relative
}

#[derive(Clone, PartialEq, Debug)]
enum Relative {
    Spouse,
    Sibling,
    Parent,
    Child,
}

struct BattleResult {
    attacker_deaths: u32,
    defender_deaths: u32,
    attacker_victor: bool,
    defender_captured: bool,
}

fn main() {

    use std::time::Instant;
    let now = Instant::now();

    let nords = CulturePrefab {
        id: Id(0),
        name: String::from("Nords"),
        language: LanguagePrefab {
            dictionary: HashMap::from([
                (String::from("birch"), String::from("borch")),
                (String::from("pine"), String::from("pin")),
                (String::from("elk"), String::from("skog")),
                (String::from("boar"), String::from("vevel")),
                (String::from("fortress"), String::from("stad")),
                (String::from("sea"), String::from("so")),
                (String::from("port"), String::from("pør")),
                (String::from("fish"), String::from("fisk")),
                (String::from("whale"), String::from("vale")),
                (String::from("kelp"), String::from("kjel")),
                (String::from("coral"), String::from("krall")),
                (String::from("scorpion"), String::from("skør")),
                (String::from("vulture"), String::from("vøl")),
                (String::from("cactus"), String::from("kak")),
                (String::from("palm"), String::from("pølm")),
            ])
        },
        first_name_male_model: MarkovChainSingleWordModel::train(vec!(
            "Alald", "Alan", "Alar", "Alarik", "Alarke", "Alarne", "Aleld", "Alen", "Alens",
            "Aler", "Alik", "Alis", "Alorn", "Asgald", "Asgan", "Asgar", "Asgarik", "Asgarke",
            "Asgarne", "Asgeld", "Asgen", "Asgens", "Asger", "Asgik", "Asgis", "Asgorn", "Bjald",
            "Bjan", "Bjar", "Bjarik", "Bjarke", "Bjarne", "Bjeld", "Bjen", "Bjens", "Bjer",
            "Bjik", "Bjis", "Bjorn", "Erald", "Eran", "Erar", "Erarik", "Erarke", "Erarne",
            "Ereld", "Eren", "Erens", "Erer", "Erik", "Eris", "Erorn", "Fenrald", "Fenran",
            "Fenrar", "Fenrarik", "Fenrarke", "Fenrarne", "Fenreld", "Fenren", "Fenrens",
            "Fenrer", "Fenrik", "Fenris", "Fenrorn", "Harald", "Haran", "Harar", "Hararik", 
            "Hararke", "Hararne", "Hareld", "Haren", "Harens", "Harer", "Harik", "Haris", 
            "Harorn", "Ingmald", "Ingman", "Ingmar", "Ingmarik", "Ingmarke", "Ingmarne", 
            "Ingmeld", "Ingmen", "Ingmens", "Ingmer", "Ingmik", "Ingmis", "Ingmorn", "Jurgald", 
            "Jurgan", "Jurgar", "Jurgarik", "Jurgarke", "Jurgarne", "Jurgeld", "Jurgen", 
            "Jurgens", "Jurger", "Jurgik", "Jurgis", "Jurgorn", "Kjald", "Kjan", "Kjar", "Kjarik", 
            "Kjarke", "Kjarne", "Kjeld", "Kjen", "Kjens", "Kjer", "Kjik", "Kjis", "Kjorn", "Mojald", 
            "Mojan", "Mojar", "Mojarik", "Mojarke", "Mojarne", "Mojeld", "Mojen", "Mojens", "Mojer", 
            "Mojik", "Mojis", "Mojorn", "Sorald", "Soran", "Sorar", "Sorarik", "Sorarke", "Sorarne", 
            "Soreld", "Soren", "Sorens", "Sorer", "Sorik", "Soris", "Sororn", "Torbald", "Torban", 
            "Torbar", "Torbarik", "Torbarke", "Torbarne", "Torbeld", "Torben", "Torbens", "Torber", 
            "Torbik", "Torbis", "Torborn", "Ulrald", "Ulran", "Ulrar", "Ulrarik", "Ulrarke", 
            "Ulrarne", "Ulreld", "Ulren", "Ulrens", "Ulrer", "Ulrik", "Ulris", "Ulrorn"
        ), 3),
        first_name_female_model: MarkovChainSingleWordModel::train(vec!(
            "Ana", "Ane", "Anen", "Ania", "Anina", "Anne", "Ante", "Beta", "Bete", "Beten",
            "Betia", "Betina", "Betne", "Bette", "Dora", "Dore", "Doren", "Doria", "Dorina",
            "Dorne", "Dorte", "Ella", "Elle", "Ellen", "Ellia", "Ellina", "Ellne", "Ellte",
            "Hana", "Hane", "Hanen", "Hania", "Hanina", "Hanne", "Hante", "Hella", "Helle",
            "Hellen", "Hellia", "Hellina", "Hellne", "Hellte", "Inga", "Inge", "Ingen", "Ingia",
            "Ingina", "Ingne", "Ingte", "Jyta", "Jyte", "Jyten", "Jytia", "Jytina", "Jytne",
            "Jytte", "Kirsta", "Kirste", "Kirsten", "Kirstia", "Kirstina", "Kirstne", "Kirstte",
            "Meta", "Mete", "Meten", "Metia", "Metina", "Metne", "Mette", "Morga", "Morge",
            "Morgen", "Morgia", "Morgina", "Morgne", "Morgte", "Silla", "Sille", "Sillen",
            "Sillia", "Sillina", "Sillne", "Sillte", "Ulla", "Ulle", "Ullen", "Ullia", "Ullina",
            "Ullne", "Ullte"
        ), 3),
        last_name_model: MarkovChainSingleWordModel::train(vec!(
            "Alaldsen", "Alansen", "Alarsen", "Alariksen", "Alarkesen", "Alarnesen", "Aleldsen",
            "Alensen", "Alenssen", "Alersen", "Aliksen", "Alissen", "Alornsen", "Asgaldsen",
            "Asgansen", "Asgarsen", "Asgariksen", "Asgarkesen", "Asgarnesen", "Asgeldsen",
            "Asgensen", "Asgenssen", "Asgersen", "Asgiksen", "Asgissen", "Asgornsen",
            "Bjaldsen", "Bjansen", "Bjarsen", "Bjariksen", "Bjarkesen", "Bjarnesen", "Bjeldsen",
            "Bjensen", "Bjenssen", "Bjersen", "Bjiksen", "Bjissen", "Bjornsen", "Eraldsen",
            "Eransen", "Erarsen", "Erariksen", "Erarkesen", "Erarnesen", "Ereldsen", "Erensen",
            "Erenssen", "Erersen", "Eriksen", "Erissen", "Erornsen", "Fenraldsen", "Fenransen",
            "Fenrarsen", "Fenrariksen", "Fenrarkesen", "Fenrarnesen", "Fenreldsen", "Fenrensen",
            "Fenrenssen", "Fenrersen", "Fenriksen", "Fenrissen", "Fenrornsen", "Haraldsen",
            "Haransen", "Hararsen", "Harariksen", "Hararkesen", "Hararnesen", "Hareldsen",
            "Harensen", "Harenssen", "Harersen", "Hariksen", "Harissen", "Harornsen",
            "Ingmaldsen", "Ingmansen", "Ingmarsen", "Ingmariksen", "Ingmarkesen", "Ingmarnesen",
            "Ingmeldsen", "Ingmensen", "Ingmenssen", "Ingmersen", "Ingmiksen", "Ingmissen",
            "Ingmornsen", "Jurgaldsen", "Jurgansen", "Jurgarsen", "Jurgariksen", "Jurgarkesen",
            "Jurgarnesen", "Jurgeldsen", "Jurgensen", "Jurgenssen", "Jurgersen", "Jurgiksen",
            "Jurgissen", "Jurgornsen", "Kjaldsen", "Kjansen", "Kjarsen", "Kjariksen", "Kjarkesen",
            "Kjarnesen", "Kjeldsen", "Kjensen", "Kjenssen", "Kjersen", "Kjiksen", "Kjissen",
            "Kjornsen", "Mojaldsen", "Mojansen", "Mojarsen", "Mojariksen", "Mojarkesen",
            "Mojarnesen", "Mojeldsen", "Mojensen", "Mojenssen", "Mojersen", "Mojiksen",
            "Mojissen", "Mojornsen", "Soraldsen", "Soransen", "Sorarsen", "Sorariksen",
            "Sorarkesen", "Sorarnesen", "Soreldsen", "Sorensen", "Sorenssen", "Sorersen",
            "Soriksen", "Sorissen", "Sorornsen", "Torbaldsen", "Torbansen", "Torbarsen",
            "Torbariksen", "Torbarkesen", "Torbarnesen", "Torbeldsen", "Torbensen", "Torbenssen",
            "Torbersen", "Torbiksen", "Torbissen", "Torbornsen", "Ulraldsen", "Ulransen",
            "Ulrarsen", "Ulrariksen", "Ulrarkesen", "Ulrarnesen", "Ulreldsen", "Ulrensen",
            "Ulrenssen", "Ulrersen", "Ulriksen", "Ulrissen", "Ulrornsen"
        ), 3)
    };

    let khajit = CulturePrefab {
        id: Id(1),
        name: String::from("Khajiit"),
        language: LanguagePrefab {
            dictionary: HashMap::from([
                (String::from("birch"), String::from("has")),
                (String::from("pine"), String::from("apa'")),
                (String::from("elk"), String::from("liz")),
                (String::from("boar"), String::from("skish")),
                (String::from("sea"), String::from("shas")),
                (String::from("fish"), String::from("rah")),
                (String::from("whale"), String::from("shin")),
                (String::from("kelp"), String::from("klash")),
                (String::from("coral"), String::from("fal")),
                (String::from("fortress"), String::from("'kanash")),
                (String::from("port"), String::from("'kapor")),
                (String::from("scorpion"), String::from("sacrah")),
                (String::from("vulture"), String::from("va'al")),
                (String::from("cactus"), String::from("kazh")),
                (String::from("palm"), String::from("pahz")),
            ])
        },
        first_name_male_model: MarkovChainSingleWordModel::train(vec!(
            "Ab'ar", "Ab'bar", "Ab'bil", "Ab'der", "Ab'dul", "Ab'gh", "Ab'ir", "Ab'kir", "Ab'med", "Ab'nir", "Ab'noud", "Ab'sien", "Ab'soud", "Ab'taba", "Ab'tabe", "Ab'urabi", "Ak'ar", "Ak'bar", "Ak'bil", "Ak'der", "Ak'dul", "Ak'gh", "Ak'ir", "Ak'kir", "Ak'med", "Ak'nir", "Ak'noud", "Ak'sien", "Ak'soud", "Ak'taba", "Ak'tabe", "Ak'urabi", "Akh'ar", "Akh'bar", "Akh'bil", "Akh'der", "Akh'dul", "Akh'gh", "Akh'ir", "Akh'kir", "Akh'med", "Akh'nir", "Akh'noud", "Akh'sien", "Akh'soud", "Akh'taba", "Akh'tabe", "Akh'urabi", "Amar", "Ambar", "Ambil", "Amder", "Amdul", "Amgh", "Amir", "Amkir", "Ammed", "Amnir", "Amnoud", "Amsien", "Amsoud", "Amtaba", "Amtabe", "Amurabi", "Fa'ar", "Fa'bar", "Fa'bil", "Fa'der", "Fa'dul", "Fa'gh", "Fa'ir", "Fa'kir", "Fa'med", "Fa'nir", "Fa'noud", "Fa'sien", "Fa'soud", "Fa'taba", "Fa'tabe", "Fa'urabi", "Husar", "Husbar", "Husbil", "Husder", "Husdul", "Husgh", "Husir", "Huskir", "Husmed", "Husnir", "Husnoud", "Hussien", "Hussoud", "Hustaba", "Hustabe", "Husurabi", "Moar", "Mobar", "Mobil", "Moder", "Modul", "Mogh", "Moir", "Mokir", "Momed", "Monir", "Monoud", "Mosien", "Mosoud", "Motaba", "Motabe", "Mourabi", "Mohamar", "Mohambar", "Mohambil", "Mohamder", "Mohamdul", "Mohamgh", "Mohamir", "Mohamkir", "Mohammed", "Mohamnir", "Mohamnoud", "Mohamsien", "Mohamsoud", "Mohamtaba", "Mohamtabe", "Mohamurabi", "Mojar", "Mojbar", "Mojbil", "Mojder", "Mojdul", "Mojgh", "Mojir", "Mojkir", "Mojmed", "Mojnir", "Mojnoud", "Mojsien", "Mojsoud", "Mojtaba", "Mojtabe", "Mojurabi", "Naar", "Nabar", "Nabil", "Nader", "Nadul", "Nagh", "Nair", "Nakir", "Named", "Nanir", "Nanoud", "Nasien", "Nasoud", "Nataba", "Natabe", "Naurabi", "Omar", "Ombar", "Ombil", "Omder", "Omdul", "Omgh", "Omir", "Omkir", "Ommed", "Omnir", "Omnoud", "Omsien", "Omsoud", "Omtaba", "Omtabe", "Omurabi", "Shaar", "Shabar", "Shabil", "Shader", "Shadul", "Shagh", "Shair", "Shakir", "Shamed", "Shanir", "Shanoud", "Shasien", "Shasoud", "Shataba", "Shatabe", "Shaurabi", "Sinar", "Sinbar", "Sinbil", "Sinder", "Sindul", "Singh", "Sinir", "Sinkir", "Sinmed", "Sinnir", "Sinnoud", "Sinsien", "Sinsoud", "Sintaba", "Sintabe", "Sinurabi", "Za'ar", "Za'bar", "Za'bil", "Za'der", "Za'dul", "Za'gh", "Za'ir", "Za'kir", "Za'med", "Za'nir", "Za'noud", "Za'sien", "Za'soud", "Za'taba", "Za'tabe", "Za'urabi", "Zan'ar", "Zan'bar", "Zan'bil", "Zan'der", "Zan'dul", "Zan'gh", "Zan'ir", "Zan'kir", "Zan'med", "Zan'nir", "Zan'noud", "Zan'sien", "Zan'soud", "Zan'taba", "Zan'tabe", "Zan'urabi",
        ), 3),
        first_name_female_model: MarkovChainSingleWordModel::train(vec!(
            "Aahin", "Aahni", "Afeliz", "Ahana", "Aheh", "Ahrazad", "Ajjan", "Akhtar", "Anita", "Araya", "Ariba", "Ashima", "Asrin", "Atima", "Azita", "Aziahin", "Aziahni", "Azifeliz", "Azihana", "Aziheh", "Azihrazad", "Azijjan", "Azikhtar", "Azinita", "Aziraya", "Aziriba", "Azishima", "Azisrin", "Azitima", "Azizita", "Elaahin", "Elaahni", "Elafeliz", "Elahana", "Elaheh", "Elahrazad", "Elajjan", "Elakhtar", "Elanita", "Elaraya", "Elariba", "Elashima", "Elasrin", "Elatima", "Elazita", "Faahin", "Faahni", "Fafeliz", "Fahana", "Faheh", "Fahrazad", "Fajjan", "Fakhtar", "Fanita", "Faraya", "Fariba", "Fashima", "Fasrin", "Fatima", "Fazita", "Khaahin", "Khaahni", "Khafeliz", "Khahana", "Khaheh", "Khahrazad", "Khajjan", "Khakhtar", "Khanita", "Kharaya", "Khariba", "Khashima", "Khasrin", "Khatima", "Khazita", "Kiahin", "Kiahni", "Kifeliz", "Kihana", "Kiheh", "Kihrazad", "Kijjan", "Kikhtar", "Kinita", "Kiraya", "Kiriba", "Kishima", "Kisrin", "Kitima", "Kizita", "Moahin", "Moahni", "Mofeliz", "Mohana", "Moheh", "Mohrazad", "Mojjan", "Mokhtar", "Monita", "Moraya", "Moriba", "Moshima", "Mosrin", "Motima", "Mozita", "Naahin", "Naahni", "Nafeliz", "Nahana", "Naheh", "Nahrazad", "Najjan", "Nakhtar", "Nanita", "Naraya", "Nariba", "Nashima", "Nasrin", "Natima", "Nazita", "Raahin", "Raahni", "Rafeliz", "Rahana", "Raheh", "Rahrazad", "Rajjan", "Rakhtar", "Ranita", "Raraya", "Rariba", "Rashima", "Rasrin", "Ratima", "Razita", "Riahin", "Riahni", "Rifeliz", "Rihana", "Riheh", "Rihrazad", "Rijjan", "Rikhtar", "Rinita", "Riraya", "Ririba", "Rishima", "Risrin", "Ritima", "Rizita", "Saahin", "Saahni", "Safeliz", "Sahana", "Saheh", "Sahrazad", "Sajjan", "Sakhtar", "Sanita", "Saraya", "Sariba", "Sashima", "Sasrin", "Satima", "Sazita", "Shaahin", "Shaahni", "Shafeliz", "Shahana", "Shaheh", "Shahrazad", "Shajjan", "Shakhtar", "Shanita", "Sharaya", "Shariba", "Shashima", "Shasrin", "Shatima", "Shazita", "Soahin", "Soahni", "Sofeliz", "Sohana", "Soheh", "Sohrazad", "Sojjan", "Sokhtar", "Sonita", "Soraya", "Soriba", "Soshima", "Sosrin", "Sotima", "Sozita", "Taahin", "Taahni", "Tafeliz", "Tahana", "Taheh", "Tahrazad", "Tajjan", "Takhtar", "Tanita", "Taraya", "Tariba", "Tashima", "Tasrin", "Tatima", "Tazita", "Zaahin", "Zaahni", "Zafeliz", "Zahana", "Zaheh", "Zahrazad", "Zajjan", "Zakhtar", "Zanita", "Zaraya", "Zariba", "Zashima", "Zasrin", "Zatima", "Zazita", 
        ), 3),
        last_name_model: MarkovChainSingleWordModel::train(vec!(
            "Abiri", "Abus", "Adavi", "Ahan", "Ahir", "Akar", "Amanni", "Amnin", "Anai", "Aoni", "Arabi", "Aspoor", "Astae", "Atani", "Avandi", "Barabiri", "Barabus", "Baradavi", "Barahan", "Barahir", "Barakar", "Baramanni", "Baramnin", "Baranai", "Baraoni", "Bararabi", "Baraspoor", "Barastae", "Baratani", "Baravandi", "Hammubiri", "Hammubus", "Hammudavi", "Hammuhan", "Hammuhir", "Hammukar", "Hammumanni", "Hammumnin", "Hammunai", "Hammuoni", "Hammurabi", "Hammuspoor", "Hammustae", "Hammutani", "Hammuvandi", "Jabiri", "Jabus", "Jadavi", "Jahan", "Jahir", "Jakar", "Jamanni", "Jamnin", "Janai", "Jaoni", "Jarabi", "Jaspoor", "Jastae", "Jatani", "Javandi", "Khabiri", "Khabus", "Khadavi", "Khahan", "Khahir", "Khakar", "Khamanni", "Khamnin", "Khanai", "Khaoni", "Kharabi", "Khaspoor", "Khastae", "Khatani", "Khavandi", "Kibiri", "Kibus", "Kidavi", "Kihan", "Kihir", "Kikar", "Kimanni", "Kimnin", "Kinai", "Kioni", "Kirabi", "Kispoor", "Kistae", "Kitani", "Kivandi", "Mahbiri", "Mahbus", "Mahdavi", "Mahhan", "Mahhir", "Mahkar", "Mahmanni", "Mahmnin", "Mahnai", "Mahoni", "Mahrabi", "Mahspoor", "Mahstae", "Mahtani", "Mahvandi", "Raibiri", "Raibus", "Raidavi", "Raihan", "Raihir", "Raikar", "Raimanni", "Raimnin", "Rainai", "Raioni", "Rairabi", "Raispoor", "Raistae", "Raitani", "Raivandi", "Robiri", "Robus", "Rodavi", "Rohan", "Rohir", "Rokar", "Romanni", "Romnin", "Ronai", "Rooni", "Rorabi", "Rospoor", "Rostae", "Rotani", "Rovandi", "Sabiri", "Sabus", "Sadavi", "Sahan", "Sahir", "Sakar", "Samanni", "Samnin", "Sanai", "Saoni", "Sarabi", "Saspoor", "Sastae", "Satani", "Savandi", "Sibiri", "Sibus", "Sidavi", "Sihan", "Sihir", "Sikar", "Simanni", "Simnin", "Sinai", "Sioni", "Sirabi", "Sispoor", "Sistae", "Sitani", "Sivandi", "Solbiri", "Solbus", "Soldavi", "Solhan", "Solhir", "Solkar", "Solmanni", "Solmnin", "Solnai", "Soloni", "Solrabi", "Solspoor", "Solstae", "Soltani", "Solvandi", "Tavakbiri", "Tavakbus", "Tavakdavi", "Tavakhan", "Tavakhir", "Tavakkar", "Tavakmanni", "Tavakmnin", "Tavaknai", "Tavakoni", "Tavakrabi", "Tavakspoor", "Tavakstae", "Tavaktani", "Tavakvandi", "Zabiri", "Zabus", "Zadavi", "Zahan", "Zahir", "Zakar", "Zamanni", "Zamnin", "Zanai", "Zaoni", "Zarabi", "Zaspoor", "Zastae", "Zatani", "Zavandi", 
        ), 3)
    };

    let regions = vec!(
        RegionPrefab {
            id: 0,
            name: String::from("Coastal"),
            elevation: (0, 0),
            temperature: (0, 5),
            soil_fertility_range: (0.8, 1.2),
            gold_generation_range: (0.8, 1.2),
            fauna: Vec::from([
                String::from("whale"),
                String::from("fish")
            ]),
            flora: Vec::from([
                String::from("kelp"),
                String::from("coral")
            ])
        },
        RegionPrefab {
            id: 1,
            name: String::from("Forest"),
            elevation: (1, 5),
            temperature: (0, 2),
            soil_fertility_range: (1.0, 1.4),
            gold_generation_range: (0.7, 1.1),
            fauna: Vec::from([
                String::from("elk"),
                String::from("boar")
            ]),
            flora: Vec::from([
                String::from("pine"),
                String::from("birch")
            ])
        },
        RegionPrefab {
            id: 2,
            name: String::from("Desert"),
            elevation: (1, 5),
            temperature: (3, 5),
            soil_fertility_range: (0.6, 1.0),
            gold_generation_range: (0.6, 1.0),
            fauna: Vec::from([
                String::from("scorpion"),
                String::from("vulture")
            ]),
            flora: Vec::from([
                String::from("cactus"),
                String::from("palm")
            ])
        },
    );

    let elapsed = now.elapsed();

    println!("");
    println!("Models created in {:.2?}", elapsed);

    let now = Instant::now();

    let mut generator = WorldHistoryGenerator::seed_world(WorldGenerationParameters {
        seed: 9563189,
        cultures: vec!(nords, khajit),
        regions
    });

    // Uncomment this for pre-generation (no rendering). Better for performance benchmmarking
    // for _ in 0..500 {
    //     generator.simulate_year();
    // }
    // let elapsed = now.elapsed();
    // println!("");
    // println!("World generated in {:.2?}", elapsed);
    // println!(" {} people", generator.world.people.len());
    // println!(" {} settlements", generator.world.settlements.len());
    // println!(" {} events", generator.world.events.len());

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl)
    };

    let mut cursor = Point2D(WORLD_MAP_WIDTH / 2, WORLD_MAP_HEIGHT / 2);

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args, &generator.world, &cursor);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);

            if generator.year < 500 {
                generator.simulate_year();
            }
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                match k.button {
                    Button::Keyboard(Key::Up) => {
                        if cursor.1 > 0 {
                            cursor.1 -= 1;
                        }
                    },
                    Button::Keyboard(Key::Down) => {
                        if cursor.1 < WORLD_MAP_HEIGHT-1 {
                            cursor.1 += 1;
                        }
                    },
                    Button::Keyboard(Key::Left) => {
                        if cursor.0 > 0 {
                            cursor.0 -= 1;
                        }
                    },
                    Button::Keyboard(Key::Right) => {
                        if cursor.0 < WORLD_MAP_WIDTH-1 {
                            cursor.0 += 1;
                        }
                    },
                    _ => (),
                }
            }
        }

    }
}

struct WorldGenerationParameters {
    seed: u32,
    cultures: Vec<CulturePrefab>,
    regions: Vec<RegionPrefab>
}

struct WorldHistoryGenerator {
    rng: Rng,
    year: u32,
    parameters: WorldGenerationParameters,
    world: WorldGraph,
    next_person_id: Id,
}

impl WorldHistoryGenerator {

    pub fn seed_world(parameters: WorldGenerationParameters) -> WorldHistoryGenerator {
        let mut rng = Rng::seeded(parameters.seed);
       
        let world_map = generate_world_map(&rng, &parameters.regions);

        let mut world = WorldGraph {
            map: world_map,
            cultures: HashMap::new(),
            factions: HistoryVec::new(),
            settlements: HistoryVec::new(),
            people: People::new(),
            events: WorldEvents::new()
        };


        let mut culture_id = Id(0);
        for culture in parameters.cultures.iter() {
            let mut culture = culture.clone();
            culture.id = culture_id.next();
            world.cultures.insert(culture.id, culture);
        }

        let mut person_id = Id(0);

        let event_date = WorldEventDate { year: 1 };

        // Generate starter people
        for _ in 0..10 {
            rng.next();
            let id = person_id.next();
            let culture = world.cultures.get(&Id(rng.randu_range(0, culture_id.seq()))).unwrap();
            let sex;
            if rng.rand_chance(0.5) {
                sex = PersonSex::Male;
            } else {
                sex = PersonSex::Female;
            }
            let faction = Faction::new(&rng, id);
            let faction_id = world.factions.insert(faction);
            let mut person = generate_person(&rng, Importance::Important, id, 1, sex, &culture, &faction_id, None);
            person.faction_relation = FactionRelation::Leader;
            world.events.push(event_date, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: person.id }));
            world.people.insert(person);
        }
        return WorldHistoryGenerator {
            rng,
            parameters,
            world,
            year: 1,
            next_person_id: person_id
        };
    }

    pub fn simulate_year(&mut self) {
        self.year = self.year + 1;
        let year = self.year;
        let event_date = WorldEventDate { year };
        println!("Year {}, {} people to process", self.year, self.world.people.inner.len());

        let mut new_people: Vec<Person> = Vec::new();

        for (_, person) in self.world.people.iter() {
            let mut person = person.borrow_mut();
            let age = (year - person.birth) as f32;
            if self.rng.rand_chance(f32::min(1.0, (age/120.0).powf(5.0))) {
                person.death = year;
                self.world.events.push(event_date, WorldEventEnum::PersonDeath(SimplePersonEvent { person_id: person.id }));
                if let Some(settlement) = person.leader_of_settlement {
                    let heirs_by_order = person.sorted_heirs();
                
                    let mut valid_heir = false;
                    for heir in heirs_by_order {
                        let mut heir = self.world.people.get_mut(&heir.person_id).unwrap();
                        if heir.alive() {
                            heir.leader_of_settlement = Some(settlement);
                            heir.importance = Importance::Important;
                            if person.faction_relation == FactionRelation::Leader {
                                heir.faction_relation = FactionRelation::Leader;
                            }
                            self.world.events.push(event_date, WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: heir.id, settlement_id: settlement }));
                            let mut faction = self.world.factions.get_mut(&heir.faction_id);
                            faction.leader = heir.id;
                            valid_heir = true;
                            break
                        }
                    }
                    if !valid_heir {
                        let culture = self.world.cultures.get(&Id(self.rng.randu_range(0, self.world.cultures.len()))).unwrap();
                        let sex;
                        if self.rng.rand_chance(0.5) {
                            sex = PersonSex::Male;
                        } else {
                            sex = PersonSex::Female;
                        }
                        self.rng.next();
                        let mut new_leader = generate_person(&self.rng, Importance::Important, self.next_person_id.next(), year, sex, &culture, &person.faction_id, None);
                        new_leader.leader_of_settlement = Some(settlement);
                        if person.faction_relation == FactionRelation::Leader {
                            new_leader.faction_relation = FactionRelation::Leader;
                        }
                        self.world.events.push(event_date, WorldEventEnum::NewSettlementLeader(NewSettlementLeaderEvent { new_leader_id: new_leader.id, settlement_id: settlement }));
                        new_people.push(new_leader);
                    }
                }

                continue
            }

            if age > 18.0 && person.spouse().is_none() && self.rng.rand_chance(0.1) {
                self.rng.next();
                let id = self.next_person_id.next();
                let spouse_age = self.rng.randu_range(18, age as usize + 10) as u32;
                let spouse_birth_year = year - u32::min(spouse_age, year);
                let culture = self.world.cultures.get(&person.culture_id).unwrap();
                let mut spouse = generate_person(&self.rng, person.importance.lower(), id, spouse_birth_year, person.sex.opposite(), culture, &person.faction_id, None);
                spouse.last_name = person.last_name.clone();
                spouse.next_of_kin.push(NextOfKin {
                    person_id: person.id,
                    relative: Relative::Spouse
                });
                person.next_of_kin.push(NextOfKin {
                    person_id: spouse.id,
                    relative: Relative::Spouse
                });
                self.world.events.push(event_date, WorldEventEnum::Marriage(MarriageEvent { person1_id: person.id, person2_id: spouse.id }));
                new_people.push(spouse.clone());
                continue;
            }

            if age > 18.0 && person.spouse().is_some() {
                let spouse = self.world.people.get_mut(person.spouse().unwrap()).unwrap();
                let couple_fertility = person.fertility(year) * spouse.fertility(year);

                if self.rng.rand_chance(couple_fertility * 0.5) {
                    let id = self.next_person_id.next();
                    self.rng.next();
                    let sex;
                    if self.rng.rand_chance(0.5) {
                        sex = PersonSex::Male;
                    } else {
                        sex = PersonSex::Female;
                    }
                    let culture = self.world.cultures.get(&person.culture_id).unwrap();
                    let mut child = generate_person(&self.rng, person.importance.lower(), id, year, sex, culture, &person.faction_id, Some(&person.last_name));
                    child.next_of_kin.push(NextOfKin { 
                        person_id: person.id,
                        relative: Relative::Parent
                    });
                    self.world.events.push(event_date, WorldEventEnum::PersonBorn(SimplePersonEvent { person_id: child.id }));
                    person.next_of_kin.push(NextOfKin { 
                        person_id: child.id,
                        relative: Relative::Child
                    });
                    new_people.push(child);
                    continue;
                }
            }

            if age > 18.0 && person.leader_of_settlement.is_none() && self.rng.rand_chance(1.0/50.0) {
                self.rng.next();
                let culture = self.world.cultures.get(&person.culture_id).unwrap();
                let settlement = generate_settlement(&self.rng, year, culture, person.faction_id, &self.world, &self.world.map, &self.parameters.regions).clone();
                let id = self.world.settlements.insert(settlement);
                self.world.events.push(event_date, WorldEventEnum::SettlementFounded(SettlementFoundedEvent { settlement_id: id, founder_id: person.id }));
                let mut faction = self.world.factions.get_mut(&person.faction_id);
                faction.settlements.insert(id);
                person.leader_of_settlement = Some(id);
                continue;
            }

            if person.faction_relation == FactionRelation::Leader {
                let faction_id = person.faction_id;
                let mut faction = self.world.factions.get_mut(&faction_id);

                if faction_id != person.faction_id {
                    panic!("{:?} {:?}", faction_id, person.faction_id);
                }

                let current_enemy = faction.relations.iter().find(|kv| *kv.1 < -0.8);

                if let Some(current_enemy) = current_enemy {
                    let chance_for_peace = 0.05;
                    if self.rng.rand_chance(chance_for_peace) {
                        let other_faction_id = current_enemy.0.clone();
                        let mut other_faction = self.world.factions.get_mut(&other_faction_id);

                        faction.relations.insert(other_faction_id, -0.2);
                        other_faction.relations.insert(faction_id, -0.2);

                        self.world.events.push(event_date, WorldEventEnum::PeaceDeclared(PeaceDeclaredEvent { faction1_id: faction_id, faction2_id: other_faction_id }));
                    }
                } else {
                    for (other_faction_id, other_faction) in self.world.factions.iter() {
                        if other_faction_id == faction_id {
                            continue
                        }
                        let opinion = faction.relations.get(&other_faction_id).unwrap_or(&0.0);
                        let chance_for_war = (*opinion * -1.0).max(0.0) * 0.001 + 0.001;
                        if self.rng.rand_chance(chance_for_war) {
                            let mut other_faction = other_faction.borrow_mut();

                            faction.relations.insert(other_faction_id, -1.0);
                            other_faction.relations.insert(faction_id, -1.0);

                            self.world.events.push(event_date, WorldEventEnum::WarDeclared(WarDeclaredEvent { faction1_id: faction_id, faction2_id: other_faction_id }));

                            break
                        }
                    }
                }
            }

        }


        for new_person in new_people {
            self.world.people.insert(new_person);
        }
        
        for (id, settlement) in self.world.settlements.iter() {
            let mut settlement = settlement.borrow_mut();
            if settlement.demographics.population <= 0 {
                continue
            }

            let settlement_tile = self.world.map.get_world_tile(settlement.xy.0, settlement.xy.1);

            // https://en.wikipedia.org/wiki/Estimates_of_historical_world_population
            let soil_fertility = settlement_tile.soil_fertility;
            let growth = self.rng.randf_range(-0.005, 0.03) + ((soil_fertility - 0.5) * 0.01);
            let child_chance = (settlement.demographics.population as f32) * growth;
            if child_chance < 0.0 {
                if child_chance > -1.0 && self.rng.rand_chance(child_chance.abs()) {
                    settlement.demographics.change_population(-1);
                } else {
                    settlement.demographics.change_population(child_chance as i32);
                }
            } else {
                if child_chance < 1.0 && self.rng.rand_chance(child_chance) {
                    settlement.demographics.population = settlement.demographics.population + 1;
                } else {
                    settlement.demographics.change_population(child_chance as i32);
                }
            }

            // Keeping an army unit posted costs 100 gold per year, for reference
            let tile_gold_range = self.parameters.regions.get(settlement_tile.region_id as usize).unwrap().gold_generation_range;
            let gold_generated = self.rng.randf_range(tile_gold_range.0, tile_gold_range.1) * settlement.demographics.population as f32;
            settlement.gold = settlement.gold + gold_generated as i32;

            // Pay current army
            let army_cost = (settlement.military.trained_soldiers * 100) + (settlement.military.conscripts * 50);
            settlement.gold = (settlement.gold - army_cost as i32).max(0);

            let army_size = settlement.military.trained_soldiers + settlement.military.conscripts;
            let army_ratio = army_size as f32 / settlement.demographics.population as f32;
            if army_ratio < 0.05 {
                let can_train = settlement.gold / 50;
                settlement.military.trained_soldiers = settlement.military.trained_soldiers + can_train  as u32;
                settlement.gold = settlement.gold - (50 * can_train);
            }
            let faction_id = settlement.faction_id;
            let mut faction = self.world.factions.get_mut(&faction_id);
            let at_war = faction.relations.iter().find(|v| *v.1 <= -0.8);
            if let Some(enemy) = at_war {
                if army_ratio < 0.05 {
                    let can_train = settlement.gold / 15;
                    settlement.military.conscripts = settlement.military.conscripts + can_train as u32;
                    settlement.gold = settlement.gold - (15 * can_train);
                }
                let siege_power = settlement.military_siege_power();
                let mut attack = None;
                if siege_power > 0.0 {
                    let enemy_faction = self.world.factions.get(enemy.0);
                    for enemy_settlement_id in enemy_faction.settlements.iter() {
                        let mut enemy_settlement = self.world.settlements.get_mut(enemy_settlement_id);
                        let defence_power = enemy_settlement.military_defence_power();
                        let power_diff = siege_power / (siege_power + defence_power);
                        let attack_chance = power_diff.powi(2);
                        if self.rng.rand_chance(attack_chance) {
                            attack = Some((enemy_settlement_id.clone(), enemy_settlement));
                        }
                    }
                }

                if let Some(enemy_settlement) = attack {
                    let battle_modifer = self.rng.randf();
                    let (enemy_settlement_id, mut enemy_settlement) = enemy_settlement;

                    let defence_power = enemy_settlement.military_defence_power();
                    let power_diff = siege_power / (siege_power + defence_power);

                    let battle_closeness = 1.0 - (battle_modifer - power_diff).abs();

                    let battle_result = BattleResult {
                        attacker_deaths: ((settlement.military.trained_soldiers + settlement.military.conscripts) as f32 * battle_closeness) as u32,
                        defender_deaths: ((enemy_settlement.military.trained_soldiers + enemy_settlement.military.conscripts) as f32 * battle_closeness) as u32,
                        attacker_victor: battle_modifer > power_diff,
                        defender_captured: battle_modifer > power_diff,
                    };

                    settlement.kill_military(battle_result.attacker_deaths, &self.rng);
                    enemy_settlement.kill_military(battle_result.defender_deaths, &self.rng);

                    let enemy_faction_id = *enemy.0;
                    let mut enemy_faction = self.world.factions.get_mut(&enemy_faction_id);

                    if battle_result.defender_captured {
                        enemy_settlement.faction_id = settlement.faction_id;
                        faction.settlements.insert(enemy_settlement_id);
                        enemy_faction.settlements.remove(&enemy_settlement_id);
                    }

                    self.world.events.push(event_date, WorldEventEnum::Siege(SiegeEvent { faction1_id: faction_id, faction2_id: enemy_faction_id, settlement1_id: id.clone(), settlement2_id: enemy_settlement_id.clone(), battle_result }));
                }



            }


        }
    }

}

fn generate_person(rng: &Rng, importance: Importance, next_id: Id, birth_year: u32, sex: PersonSex, culture: &CulturePrefab, faction: &Id, surname: Option<&str>) -> Person {
    let rng = rng.derive("person");
    let first_name;
    match sex {
        PersonSex::Male => first_name = culture.first_name_male_model.generate(&rng.derive("first_name"), 4, 15),
        PersonSex::Female => first_name = culture.first_name_female_model.generate(&rng.derive("first_name"), 4, 15)
    }
    let first_name = Strings::capitalize(&first_name);
    let last_name;
    match surname {
        Some(str) => last_name = String::from(str),
        None => last_name = Strings::capitalize(&culture.last_name_model.generate(&rng.derive("last_name"), 4, 15))
    }
    return Person {
        id: next_id,
        importance,
        first_name,
        last_name: last_name.clone(),
        birth_last_name: last_name.clone(),
        birth: birth_year,
        sex,
        culture_id: culture.id,
        faction_id: faction.clone(),
        faction_relation: FactionRelation::Member,
        death: 0,
        next_of_kin: Vec::new(),
        leader_of_settlement: None
    }
}


fn generate_settlement(rng: &Rng, founding_year: u32, culture: &CulturePrefab, faction: Id, world_graph: &WorldGraph, world_map: &WorldMap, regions: &Vec<RegionPrefab>) -> Settlement {
    let mut rng = rng.derive("settlement");
    let mut xy = Point2D(0, 0);
    // TODO: What if there's no more places?
    'candidates: for _ in 1..10 {
        xy = Point2D(rng.randu_range(0, WORLD_MAP_WIDTH), rng.randu_range(0, WORLD_MAP_HEIGHT));
        for (_, settlement) in world_graph.settlements.iter() {
            if settlement.borrow().xy.dist_squared(&xy) < 5.0_f32.powi(2) {
                continue 'candidates;
            }
        }
        break;
    }
    let region_id = world_map.get_world_tile(xy.0, xy.1).region_id as usize;
    let region = regions.get(region_id).unwrap();

    return SettlementBuilder::colony(&rng, xy, founding_year, culture, faction, region).create()
}

fn generate_world_map(rng: &Rng, regions: &Vec<RegionPrefab>) -> WorldMap {
    let rng = rng.derive("world_map");
    let mut map = WorldMap {
        elevation: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
        temperature: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
        soil_ferility: [0.0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
        region_id: [0; WORLD_MAP_HEIGHT * WORLD_MAP_WIDTH],
    };
    let n_elev = Perlin::new(rng.derive("elevation").seed());
    let n_temp = Perlin::new(rng.derive("temperature").seed());
    let n_reg = Perlin::new(rng.derive("region").seed());
    let n_fert = Perlin::new(rng.derive("fertility").seed());
    for y in 0..WORLD_MAP_HEIGHT {
        for x in 0..WORLD_MAP_WIDTH {
            let i = (y * WORLD_MAP_WIDTH) + x;
            let xf = x as f64;
            let yf = y as f64;
            {
                let low = n_elev.get([xf / 10.0, yf / 10.0]);
                let med = n_elev.get([xf / 4.0, yf / 4.0]);
                map.elevation[i] = ((1.0+low+med) / 4.0 * 5.0) as u8;
            }
            {
                let low = n_temp.get([xf / 10.0, yf / 10.0]);
                // let med = n_temp.get([xf / 4.0, yf / 4.0]);
                map.temperature[i] = (low * 5.0) as u8;
            }
            {
                let mut region_candidates: Vec<u8> = Vec::new();
                for (j, region) in regions.iter().enumerate() {
                    if map.elevation[i] >= region.elevation.0 && map.elevation[i] <= region.elevation.1 && map.temperature[i] >= region.temperature.0 && map.temperature[i] <= region.temperature.1 {
                        region_candidates.push(j as u8);
                    }
                }
                match region_candidates.len() {
                    0 => panic!("No region candidate for elevation {} and temperature {}", map.elevation[i], map.temperature[i]),
                    1 => map.region_id[i] = region_candidates.pop().expect("Already checked"),
                    _ => {
                        let noise = n_reg.get([xf / 10.0, yf / 10.0]);
                        map.region_id[i] = region_candidates[(noise * region_candidates.len() as f64) as usize];
                    }
                }
            }
            {
                let region_fertility_range = regions[map.region_id[i] as usize].soil_fertility_range;
                let noise_modif = n_fert.get([xf / 10.0, yf / 10.0]) as f32;
                let noise_modif = (noise_modif + 1.0) / 2.0;
                map.soil_ferility[i] = noise_modif * (region_fertility_range.1 - region_fertility_range.0) + region_fertility_range.0;
            }
        }
    }
    return map;
}