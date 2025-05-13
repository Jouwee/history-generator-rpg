extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;


use std::{time::Instant, vec};
use commons::{markovchains::MarkovChainSingleWordModel, rng::Rng};
use engine::{asset::assets::Assets, audio::{Audio, SoundFile, TrackMood}, debug::overlay::DebugOverlay, geometry::Coord2, gui::tooltip::TooltipRegistry, input::{InputEvent, InputState}, render::RenderContext, scene::{Scene, Update}, Color};
use game::{actor::actor::Actor, chunk::Chunk, factory::item_factory::ItemFactory, options::GameOptions, GameSceneState, InputEvent as OldInputEvent};
use resources::resources::Resources;
use world::{event::*, history_generator::WorldGenerationParameters, item::Item, worldgen::WorldGenScene};

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::{event_loop::{EventSettings, Events}, ButtonArgs, UpdateArgs};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::input::{Button, ButtonState, Key};
use piston::ButtonEvent;
use piston::MouseCursorEvent;
use piston::window::WindowSettings;

pub(crate) mod engine;
pub(crate) mod commons;
pub(crate) mod chunk_gen;
pub(crate) mod globals;
pub(crate) mod resources;
pub(crate) mod world;
pub(crate) mod game;

enum SceneEnum {
    None,
    WorldGen(WorldGenScene),
    Game(GameSceneState)
}

pub(crate) struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    context: GameContext,
    scene: SceneEnum,
    debug_overlay: DebugOverlay,
    display_context: DisplayContext
}

pub(crate) struct GameContext {
    audio: Audio,
    assets: Assets,
    resources: Resources,
    tooltips: TooltipRegistry,
    display_context: DisplayContext
}

pub(crate) struct DisplayContext {
    pub(crate) scale: f64,
    pub(crate) camera_rect: [f64; 4],
    pub(crate) gui_rect: [f64; 4],
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let texture_settings = TextureSettings::new().filter(Filter::Nearest);
        let mut glyphs = GlyphCache::new("./assets/alagard.ttf", (), texture_settings).expect("Could not load font");
        let mut small_glyphs = GlyphCache::new("./assets/enter-the-gungeon-small.ttf", (), texture_settings).expect("Could not load font");


        let c = self.gl.draw_begin(args.viewport());
        
        // Clear the screen.
        clear(Color::from_hex("090714").f32_arr(), &mut self.gl);
        let mut context = RenderContext {
            context: c,
            layout_rect: [0., 0., args.viewport().window_size[0], args.viewport().window_size[1]],
            camera_rect: [0., 0., args.viewport().window_size[0], args.viewport().window_size[1]],
            transform_queue: vec!(c.transform.clone()),
            gl: &mut self.gl,
            default_font: &mut glyphs,
            small_font: &mut small_glyphs,
            textures: Vec::new(),
        };
        match &mut self.scene {
            SceneEnum::None => {},
            SceneEnum::WorldGen(game_state) => {
                game_state.render(&mut context, &mut self.context);
            },
            SceneEnum::Game(game_state) => {
                game_state.render(&mut context, &mut self.context);
            },
        }
        self.debug_overlay.render(&mut context);
        // TODO: This is really disconnected
        self.display_context.camera_rect = context.camera_rect;
        self.display_context.gui_rect = context.layout_rect;
        self.context.display_context.camera_rect = context.camera_rect;
        self.context.display_context.gui_rect = context.layout_rect;
        self.gl.draw_end();

    }

    fn update(&mut self, args: &UpdateArgs, event_settings: &EventSettings, last_mouse_pos: [f64; 2]) {
        let mut update = Update {
            delta_time: 0.,
            max_update_time: (1. / event_settings.ups as f64),
            mouse_pos_cam: [0., 0.],
            mouse_pos_gui: [0., 0.]
        };
        update.delta_time = args.dt;
        let p = last_mouse_pos;
        update.mouse_pos_cam = [p[0] / self.display_context.scale + self.display_context.camera_rect[0], p[1] / self.display_context.scale + self.display_context.camera_rect[1]];
        update.mouse_pos_gui = [p[0] / self.display_context.scale, p[1] / self.display_context.scale];

        self.context.audio.update(&update);
        self.debug_overlay.update(&update);
        match &mut self.scene {
            SceneEnum::None => {},
            SceneEnum::WorldGen(game_state) => {
                game_state.update(&update, &mut self.context);
            },
            SceneEnum::Game(game_state) => {
                game_state.update(&update, &mut self.context);
            },
        }
    }

    fn input(&mut self, args: &OldInputEvent) {
        self.debug_overlay.input(args);
        match &mut self.scene {
            SceneEnum::None => {},
            SceneEnum::WorldGen(game_state) => {
                game_state.input(args, &mut self.context);
            },
            SceneEnum::Game(game_state) => {
                game_state.input(args, &mut self.context);
            },
        }
    }
}

fn main() {

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [200, 200])
        .graphics_api(opengl)
        .build()
        .unwrap();

    let resources = Resources::new();

    let tooltips = TooltipRegistry::new();

    let options = GameOptions {
        audio: game::options::AudioOptions { music_volume: 0.0 }
    };

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        context: GameContext {
            audio: Audio::new(options.audio.clone()),
            assets: Assets::new(),
            resources,
            tooltips,
            display_context: DisplayContext {
                scale: 2.,
                camera_rect: [0.; 4],
                gui_rect: [0.; 4]
            }
        },
        scene: SceneEnum::None,
        debug_overlay: DebugOverlay::new(),
        display_context: DisplayContext {
            scale: 2.,
            camera_rect: [0.; 4],
            gui_rect: [0.; 4]
        }
    };
    app.context.resources.load();

    app.context.audio.register_track(TrackMood::Regular, SoundFile::new("tracks/fantasy-music-lumina-143991.mp3"));
    app.context.audio.register_track(TrackMood::Regular, SoundFile::new("tracks/forgotten-land-epic-dark-fantasy-195835.mp3"));
    app.context.audio.register_track(TrackMood::Regular, SoundFile::new("tracks/the-spell-dark-magic-background-music-ob-lix-8009.mp3"));
    app.context.audio.register_track(TrackMood::Battle, SoundFile::new("tracks/cinematic-battle-music-271343.mp3"));
    app.context.audio.register_track(TrackMood::Battle, SoundFile::new("tracks/fantasy-pagan-medieval-cinematic-epic-war-battle-119770.mp3"));

    app.scene = SceneEnum::WorldGen(WorldGenScene::new(WorldGenerationParameters {
            seed: 1234567
        }, &app.context.resources));

    if let SceneEnum::WorldGen(scene) = &mut app.scene {
        scene.init(&mut app.context);
    }

    let mut last_mouse_pos = [0.0, 0.0];

    let mut event_settings = EventSettings::new();
    event_settings.max_fps = 30;
    event_settings.ups = 30;

    let mut input_state = InputState::new();

    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            let now: Instant = Instant::now();
            app.render(&args);
            app.debug_overlay.render_time(now.elapsed());
        }

        if let Some(args) = e.update_args() {
            let now: Instant = Instant::now();
            app.update(&args, &event_settings, last_mouse_pos);
            app.debug_overlay.update_time(now.elapsed());
        }

        if let Some(k) = e.mouse_cursor_args() {
            last_mouse_pos = k;
            // TODO: Fake event
            let b = ButtonArgs { state: ButtonState::Release, button: Button::Keyboard(Key::AcBookmarks), scancode: None };
            let input_event = OldInputEvent {
                mouse_pos_cam: [k[0] / app.display_context.scale + app.display_context.camera_rect[0], k[1] / app.display_context.scale + app.display_context.camera_rect[1]],
                mouse_pos_gui: [k[0] / app.display_context.scale, k[1] / app.display_context.scale],
                button_args: b,
                evt: InputEvent::from_mouse_move(k, &app.display_context, &mut input_state)
            };
            app.input(&input_event);
        }

        if let Some(k) = e.button_args() {
            let now: Instant = Instant::now();
            if k.state == ButtonState::Press || k.state == ButtonState::Release {
                let p = last_mouse_pos;
                let input_event = OldInputEvent {
                    mouse_pos_cam: [p[0] / app.display_context.scale + app.display_context.camera_rect[0], p[1] / app.display_context.scale + app.display_context.camera_rect[1]],
                    mouse_pos_gui: [p[0] / app.display_context.scale, p[1] / app.display_context.scale],
                    button_args: k,
                    evt: InputEvent::from_button_args(&k, &mut input_state)
                };

                app.input(&input_event);

                if let Button::Keyboard(Key::Return) = k.button {
                    if let SceneEnum::WorldGen(scene) = app.scene {
                        let world = scene.into_world();
                        world.dump_events("lore.log", &app.context.resources);

                        let species_id = app.context.resources.species.id_of("species:human");
                        let species = app.context.resources.species.get(&species_id);
                        let mut player = Actor::player(Coord2::xy(16, 16), &species_id, species);

                        let mut rng = Rng::seeded("player");

                        player.inventory.add(ItemFactory::weapon(&mut rng, &app.context.resources).make());
                        player.inventory.add(ItemFactory::weapon(&mut rng, &app.context.resources).make());

                        player.inventory.equip(1);

                        let cursor = Coord2::xy(128, 128);
                        let chunk = Chunk::from_world_tile(&world, &app.context.resources, cursor, player);
                        let mut scene = GameSceneState::new(world, cursor, chunk);
                        scene.init(&mut app.context);
                        app.scene = SceneEnum::Game(scene);

                        continue
                    }
                }

                if let Button::Keyboard(Key::F4) = k.button {
                    if let SceneEnum::Game(scene) = app.scene {
                        let chunk = Chunk::playground(&app.context.resources, scene.chunk.player);
                        let mut scene = GameSceneState::new(scene.world, scene.world_pos, chunk);
                        scene.init(&mut app.context);
                        app.scene = SceneEnum::Game(scene);
                        continue
                    }
                }

            }
            app.debug_overlay.input_time(now.elapsed());

        }

    }
}
