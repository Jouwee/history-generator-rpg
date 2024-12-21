extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;


use std::{collections::HashMap, fs::File, vec, io::Write};
use commons::{history_vec::Id, markovchains::MarkovChainSingleWordModel};
use engine::{assets::Assets, debug::overlay::DebugOverlay, geometry::Coord2, render::RenderContext, scene::{Scene, Update}, Color};
use game::{actor::Actor, chunk::Chunk, codex::knowledge_codex::KnowledgeCodex, GameSceneState, InputEvent};
use literature::biography::BiographyWriter;
use world::{culture::{Culture, LanguagePrefab}, event::*, history_generator::WorldGenerationParameters, item::{Item, Lance, Mace, Sword}, person::{Person, Relative}, region::Region, world::World, world_scene::WorldScene, worldgen::WorldGenScene};

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::input::{Button, ButtonState, Key};
use piston::ButtonEvent;
use piston::MouseCursorEvent;
use piston::window::WindowSettings;

pub mod engine;
pub mod commons;
pub mod literature;
pub mod world;
pub mod game;

enum SceneEnum {
    WorldGen(WorldGenScene),
    World(WorldScene),
    Game(GameSceneState)
}

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    scene: SceneEnum,
    assets: Assets,
    debug_overlay: DebugOverlay
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        let mut glyphs = GlyphCache::new("./assets/Minecraft.ttf", (), texture_settings).expect("Could not load font");


        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(Color::from_hex("14233a").f32_arr(), gl);
            let mut context = RenderContext {
                args,
                context: c,
                layout_rect: [0., 0., c.get_view_size()[0], c.get_view_size()[1]],
                original_transform: c.transform.clone(),
                gl,
                assets: &mut self.assets,
                default_font: &mut glyphs
            };
            match &mut self.scene {
                SceneEnum::WorldGen(game_state) => {
                    game_state.render(&mut context);
                },
                SceneEnum::World(game_state) => {
                    game_state.render(&mut context);
                },
                SceneEnum::Game(game_state) => {
                    game_state.render(&mut context);
                },
            }
            self.debug_overlay.render(&mut context);
        });
    }

    fn update(&mut self, args: &Update) {
        self.debug_overlay.update(args);
        match &mut self.scene {
            SceneEnum::WorldGen(game_state) => {
                game_state.update(args);
            },
            SceneEnum::World(game_state) => {
                game_state.update(args);
            },
            SceneEnum::Game(game_state) => {
                game_state.update(args);
            },
        }
    }

    fn input(&mut self, args: &InputEvent) {
        self.debug_overlay.input(args);
        match &mut self.scene {
            SceneEnum::WorldGen(game_state) => {
                game_state.input(args);
            },
            SceneEnum::World(game_state) => {
                game_state.input(args);
            },
            SceneEnum::Game(game_state) => {
                game_state.input(args);
            },
        }
    }
}

fn main() {

    use std::time::Instant;
    let now = Instant::now();

    let nords = Culture {
        id: Id(0),
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

    let khajit = Culture {
        id: Id(1),
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
        Region {
            id: 0,
            name: String::from("Coastal"),
            elevation: (-2000, 0),
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
        Region {
            id: 1,
            name: String::from("Coastal"),
            elevation: (0, 16),
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
        Region {
            id: 2,
            name: String::from("Forest"),
            elevation: (16, 255),
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
        Region {
            id: 3,
            name: String::from("Desert"),
            elevation: (16, 255),
            temperature: (3, 6),
            soil_fertility_range: (0.5, 0.9),
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
        gl: GlGraphics::new(opengl),
        scene: SceneEnum::WorldGen(WorldGenScene::new(WorldGenerationParameters {
            seed: 1234567,
            cultures: vec!(nords, khajit),
            regions: regions,            
            great_beasts_yearly_spawn_chance: 0.1
        })),
        assets: Assets::new(),
        debug_overlay: DebugOverlay::new()
    };


    let mut last_mouse_pos = [0.0, 0.0];

    let mut event_settings = EventSettings::new();
    event_settings.max_fps = 30;
    event_settings.ups = 30;

    let mut update = Update {
        delta_time: 0.,
        max_update_time: (1. / event_settings.ups as f64),
        updates_per_second: event_settings.ups as u32,
    };

    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            update.delta_time = args.dt;
            app.update(&update);
        }

        if let Some(k) = e.mouse_cursor_args() {
            last_mouse_pos = k;
        }

        if let Some(k) = e.button_args() {
            if k.state == ButtonState::Press {
                let input_event = InputEvent { mouse_pos: last_mouse_pos, button_args: k };
                app.input(&input_event);

                if let Button::Keyboard(Key::Return) = k.button {
                    if let SceneEnum::WorldGen(scene)   = app.scene {
                        let world = scene.into_world();
                        // Dumps the world history into a file
                        let mut f = File::create("history.log").unwrap();
                        let writer = BiographyWriter::new(&world);
                        for event in world.events.iter() {
                            writeln!(&mut f, "{}", writer.event(event)).unwrap();
                        }
                        let species = world.species.get(&Id(0)).unwrap();
                        let mut player = Actor::player(Coord2::xy(32, 32), species);

                        player.inventory.add(Item::Sword(Sword::new(Id(3), Id(0), Id(1), Id(1), &world)));
                        player.inventory.add(Item::Mace(Mace::new(Id(3), Id(0), Id(1), &world)));
                        player.inventory.add(Item::Lance(Lance::new(Id(3), Id(0), &world)));
                        let codex = KnowledgeCodex::new();
                        app.scene = SceneEnum::World(WorldScene::new(world, player, codex));
                        continue
                    }
                }

                if let Button::Keyboard(Key::Return) = k.button {
                    if let SceneEnum::World(scene) = app.scene {
                        let chunk = Chunk::from_world_tile(&scene.world, scene.cursor);
                        app.scene = SceneEnum::Game(GameSceneState::new(scene.world, scene.player, scene.codex, chunk));
                        continue
                    }
                }

                if let Button::Keyboard(Key::W) = k.button {
                    if let SceneEnum::Game(scene) = app.scene {
                        // TODO:
                        // let killed_npcs = scene.chunk.killed_people;
                        // for id in killed_npcs {
                            // let mut person = scene.world.people.get_mut(&id).unwrap();
                            // person.death = scene.year;
                            // scene.world.events.push(WorldEventDate { year: generator.year }, WorldEventEnum::PersonDeath(SimplePersonEvent { person_id: person.id }));
                            // TODO: Inheritance
                        // }
                        app.scene = SceneEnum::World(WorldScene::new(scene.world, scene.player, scene.codex));
                        continue
                    }
                }
                
            }
        }

    }
}
