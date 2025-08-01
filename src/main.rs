use std::{time::Instant, vec};
use commons::{markovchains::MarkovChainSingleWordModel, rng::Rng};
use engine::{asset::assets::Assets, audio::{Audio, SoundFile, TrackMood}, debug::overlay::DebugOverlay, geometry::Coord2, gui::tooltip::TooltipRegistry, input::{InputEvent, InputState}, render::RenderContext, scene::{Scene, Update}, Color};
use game::{actor::actor::Actor, chunk::Chunk, factory::item_factory::ItemFactory, inventory::inventory::EquipmentType, options::GameOptions, GameSceneState, InputEvent as OldInputEvent};
use resources::resources::Resources;
use sdl2_window::Sdl2Window;
use world::{event::*, history_generator::WorldGenerationParameters, item::Item, worldgen::WorldGenScene};

use opengl_graphics::{GlGraphics, OpenGL};
use piston::{event_loop::{EventSettings, Events}, ButtonArgs, EventLoop, UpdateArgs};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::input::{Button, ButtonState, Key};
use piston::ButtonEvent;
use piston::MouseCursorEvent;
use piston::window::WindowSettings;

use crate::{chunk_gen::chunk_generator::ChunkLayer, engine::geometry::Size2D};

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
    sprite_i: usize,
    sprite_c: f64,
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
    display_context: DisplayContext,
    drag_item: Option<Item>
}

pub(crate) struct DisplayContext {
    pub(crate) scale: f64,
    pub(crate) camera_rect: [f64; 4],
    pub(crate) gui_rect: [f64; 4],
}

pub(crate) const SPRITE_FPS: f64 = 1. / 16.;

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let c = self.gl.draw_begin(args.viewport());
        
        // Clear the screen.
        clear(Color::from_hex("090714").f32_arr(), &mut self.gl);
        let mut context = RenderContext {
            context: c,
            layout_rect: [0., 0., args.viewport().window_size[0], args.viewport().window_size[1]],
            camera_rect: [0., 0., args.viewport().window_size[0], args.viewport().window_size[1]],
            transform_queue: vec!(c.transform.clone()),
            gl: &mut self.gl,
            textures: Vec::new(),
            sprite_i: self.sprite_i
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
        self.debug_overlay.render(&mut context, &mut self.context);
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
        };
        update.delta_time = args.dt;
        let p = last_mouse_pos;
        update.mouse_pos_cam = [p[0] / self.display_context.scale + self.display_context.camera_rect[0], p[1] / self.display_context.scale + self.display_context.camera_rect[1]];

        self.sprite_c += args.dt;
        if self.sprite_c > SPRITE_FPS {
            self.sprite_i += 1;
            self.sprite_c -= SPRITE_FPS;
        }

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

    let mut window: Sdl2Window =
        WindowSettings::new("Tales of Kathay", [1024, 768])
            // .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();

    let resources = Resources::new();

    let tooltips = TooltipRegistry::new();

    let options = GameOptions {
        audio: game::options::AudioOptions { music_volume: 0.0 }
    };

    let gl = GlGraphics::new(opengl);
    
    // Create a new game and run it.
    let mut app = App {
        gl,
        context: GameContext {
            audio: Audio::new(options.audio.clone()),
            assets: Assets::new(),
            resources,
            tooltips,
            display_context: DisplayContext {
                scale: 2.,
                camera_rect: [0.; 4],
                gui_rect: [0.; 4]
            },
            drag_item: None,
        },
        sprite_i: 0,
        sprite_c: 0.,
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
        seed: 1234567,
        world_size: Size2D(64, 48),
        history_length: 5000,
        number_of_seed_cities: 20,
        seed_cities_population: 20,
        num_plate_tectonics: 5,
        st_strength: 1.0,
        st_city_population: 50,
    }, &app.context.resources));

    if let SceneEnum::WorldGen(scene) = &mut app.scene {
        scene.init(&mut app.context);
    }

    let mut last_mouse_pos = [0.0, 0.0];

    let mut event_settings = EventSettings::new();
    event_settings.set_max_fps(1000);
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
                button_args: b,
                evt: InputEvent::from_mouse_move(k, &app.display_context, &mut input_state)
            };
            app.input(&input_event);
        }

        if let Some(k) = e.button_args() {
            let now: Instant = Instant::now();
            if k.state == ButtonState::Press || k.state == ButtonState::Release {
                let input_event = OldInputEvent {
                    button_args: k,
                    evt: InputEvent::from_button_args(&k, &mut input_state)
                };

                app.input(&input_event);

                if let Button::Keyboard(Key::Return) = k.button {
                    if let SceneEnum::WorldGen(scene) = app.scene {
                        let mut world = scene.into_world();
                        world.find_goal(&mut app.context.resources);
                        world.dump_events("lore.log", &app.context.resources);

                        let species_id = app.context.resources.species.id_of("species:human");
                        let species = app.context.resources.species.get(&species_id);
                        let mut player = Actor::player(Coord2::xy(16, 16), &species_id, species);

                        let mut rng = Rng::seeded("player");

                        let _ = player.inventory.add(ItemFactory::weapon(&mut rng, &app.context.resources).make());
                        let _ = player.inventory.add(ItemFactory::torso_garment(&mut rng, &app.context.resources));
                        let _ = player.inventory.add(ItemFactory::inner_armor(&mut rng, &app.context.resources));
                        let _ = player.inventory.add(ItemFactory::boots(&mut rng, &app.context.resources));
                        let _ = player.inventory.add(ItemFactory::pants(&mut rng, &app.context.resources));
                        let _ = player.inventory.add(ItemFactory::weapon(&mut rng, &app.context.resources).make());
                        let _ = player.inventory.add(ItemFactory::weapon(&mut rng, &app.context.resources).make());

                        player.inventory.auto_equip();

                        let cursor = Coord2::xy(world.map.size.0 as i32 / 2, world.map.size.1 as i32 / 2);
                        let chunk = Chunk::from_world_tile(&world, &app.context.resources, cursor, ChunkLayer::Surface, player);
                        let mut scene = GameSceneState::new(world, cursor, chunk);
                        scene.init(&mut app.context);
                        app.scene = SceneEnum::Game(scene);

                        continue
                    }
                }

                if let Button::Keyboard(Key::F4) = k.button {
                    if let SceneEnum::Game(scene) = app.scene {
                        let chunk = Chunk::playground(&app.context.resources, scene.chunk.player().clone(), &scene.world);
                        let mut scene = GameSceneState::new(scene.world, scene.world_pos, chunk);
                        scene.init(&mut app.context);
                        app.scene = SceneEnum::Game(scene);
                        continue
                    }
                }

                if let Button::Keyboard(Key::F5) = k.button {
                    crate::engine::assets::assets().reload_all();
                }
                

            }
            app.debug_overlay.input_time(now.elapsed());

        }

    }
}
