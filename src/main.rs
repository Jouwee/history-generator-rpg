#![windows_subsystem = "windows"]

use std::{ops::ControlFlow, time::Instant, vec};
use commons::{markovchains::MarkovChainSingleWordModel, rng::Rng};
use engine::{audio::Audio, debug::overlay::DebugOverlay, geometry::Coord2, gui::tooltip::TooltipRegistry, input::{InputEvent, InputState}, render::RenderContext, scene::{Scene, Update}, Color};
use game::{actor::actor::Actor, factory::item_factory::ItemFactory, inventory::inventory::EquipmentType, options::GameOptions, GameSceneState};
use glutin_window::GlutinWindow;
use resources::resources::Resources;
use world::{event::*, history_generator::WorldGenerationParameters, item::Item, worldgen::WorldGenScene};

use opengl_graphics::{GlGraphics, OpenGL};
use piston::{event_loop::{EventSettings, Events}, EventLoop, MouseScrollEvent, UpdateArgs};
use piston::input::{RenderArgs, RenderEvent, UpdateEvent};
use piston::input::{Button, ButtonState, Key};
use piston::ButtonEvent;
use piston::MouseCursorEvent;
use piston::window::{Window, WindowSettings};

use crate::{engine::{geometry::Size2D, scene::BusEvent}, game::{chunk::{ChunkCoord, ChunkLayer}, state::{AiGroups, GameState}}, loadsave::LoadSaveManager, world::main_menu::{MainMenuOption, MainMenuScene}};

pub(crate) mod commons;
pub(crate) mod chunk_gen;
pub(crate) mod engine;
pub(crate) mod game;
pub(crate) mod globals;
pub(crate) mod loadsave;
pub(crate) mod localization;
pub(crate) mod resources;
pub(crate) mod world;

enum SceneEnum {
    None,
    MainMenu(MainMenuScene),
    WorldGen(WorldGenScene),
    Game(GameSceneState)
}

pub(crate) struct App {
    window: GlutinWindow,
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
    resources: Resources,
    tooltips: TooltipRegistry,
    display_context: DisplayContext,
    drag_item: Option<Item>,
    event_bus: Vec<BusEvent>
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
            SceneEnum::MainMenu(game_state) => {
                game_state.render(&mut context, &mut self.context);
            },
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
            SceneEnum::MainMenu(game_state) => {
                game_state.update(&update, &mut self.context);
            },
            SceneEnum::WorldGen(game_state) => {
                game_state.update(&update, &mut self.context);
            },
            SceneEnum::Game(game_state) => {
                game_state.update(&update, &mut self.context);
            },
        }
    }

    fn input(&mut self, args: &InputEvent) {
        self.debug_overlay.input(&args);
        match &mut self.scene {
            SceneEnum::None => {},
            SceneEnum::MainMenu(game_state) => {
                match game_state.input(args, &mut self.context) {
                    ControlFlow::Break(MainMenuOption::NewGame) => {
                        self.scene = SceneEnum::WorldGen(WorldGenScene::new(WorldGenerationParameters {
                            seed: Rng::rand().rand_u32(),
                            world_size: Size2D(64, 48),
                            history_length: 2000,
                            number_of_seed_cities: 3,
                            seed_cities_population: 15,
                            num_plate_tectonics: 5,
                            st_strength: 1.0,
                            st_city_count: 7,
                            st_city_population: 20,
                            st_village_count: 20,
                            st_village_population: 10,
                        }, &self.context.resources));
                    },
                    ControlFlow::Break(MainMenuOption::LoadGame) => {
                        let load_save_manager = LoadSaveManager::new();
                        // TODO(ROO4JcDl): Unwrap
                        let mut world = load_save_manager.load_world().unwrap();

                        // TODO(ROO4JcDl): These should be stored
                        let (creature_id, pos) = world.create_scenario().expect("No playable scenario found");

                        let creature = world.creatures.get(&creature_id);
                        let species = self.context.resources.species.get(&creature.species);
                        let mut player = Actor::from_creature(Coord2::xy(16, 16), AiGroups::player(), creature_id, &creature, &creature.species, &species, &world, &self.context.resources);
                        drop(creature);

                        let mut rng = Rng::seeded(creature_id).derive("equipment");
                        let _ = player.inventory.add(ItemFactory::starter_weapon(&mut rng, &self.context.resources).make());

                        player.inventory.auto_equip(&self.context.resources);

                        let chunk = GameState::from_world_tile(&world, &self.context.resources, ChunkCoord::new(pos, ChunkLayer::Surface), player);
                        let mut scene = GameSceneState::new(world, chunk);
                        scene.init(&mut self.context);
                        self.scene = SceneEnum::Game(scene);
                    }
                    ControlFlow::Break(MainMenuOption::Quit) => self.window.set_should_close(true),
                    _ => ()
                }
            },
            SceneEnum::WorldGen(game_state) => {
                let _ = game_state.input(args, &mut self.context);
            },
            SceneEnum::Game(game_state) => {
                let _ = game_state.input(args, &mut self.context);
            },
        }
    }

    fn event(&mut self, event: &BusEvent) {
        match &mut self.scene {
            SceneEnum::None => {},
            SceneEnum::MainMenu(game_state) => {
                let _ = game_state.event(event, &mut self.context);
            },
            SceneEnum::WorldGen(game_state) => {
                let _ = game_state.event(event, &mut self.context);
            },
            SceneEnum::Game(game_state) => {
                let _ = game_state.event(event, &mut self.context);
            },
        }
    }
}

fn main() {

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    let window = GlutinWindow::new(
        &WindowSettings::new("Tales of Kathay", [1024, 768]).graphics_api(opengl)
    );
    let window = match window {
        Err(err) => {
            fatal!("{err}");
            panic!("Failed to create Window. Check logs.");
        },
        Ok(window) => window
    };

    let resources = Resources::new();

    let tooltips = TooltipRegistry::new();

    let options = GameOptions {
        audio: game::options::AudioOptions { music_volume: 0.0 }
    };

    let gl = GlGraphics::new(opengl);
    
    // Create a new game and run it.
    let mut app = App {
        gl,
        window,
        context: GameContext {
            audio: Audio::new(options.audio.clone()),
            resources,
            tooltips,
            display_context: DisplayContext {
                scale: 2.,
                camera_rect: [0.; 4],
                gui_rect: [0.; 4]
            },
            drag_item: None,
            event_bus: Vec::new(),
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

    app.scene = SceneEnum::MainMenu(MainMenuScene::new());

    if let SceneEnum::WorldGen(scene) = &mut app.scene {
        scene.init(&mut app.context);
    }

    let mut last_mouse_pos = [0.0, 0.0];

    let mut event_settings = EventSettings::new();
    event_settings.set_max_fps(60);
    event_settings.set_ups(30);

    let mut input_state = InputState::new();

    let mut events = Events::new(event_settings);
    while let Some(e) = events.next(&mut app.window) {
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
            let now: Instant = Instant::now();
            last_mouse_pos = k;
            let input_event = InputEvent::from_mouse_move(k, &app.display_context, &mut input_state);
            app.input(&input_event);
            app.debug_overlay.input_time(now.elapsed());
        }

        if let Some(k) = e.mouse_scroll_args() {
            let now: Instant = Instant::now();
            last_mouse_pos = k;
            let input_event = InputEvent::from_mouse_scroll(k, &mut input_state);
            app.input(&input_event);
            app.debug_overlay.input_time(now.elapsed());
        }

        if let Some(k) = e.button_args() {
            let now: Instant = Instant::now();
            if k.state == ButtonState::Press || k.state == ButtonState::Release {
                let input_event = InputEvent::from_button_args(&k, &mut input_state);

                app.input(&input_event);

                if let Button::Keyboard(Key::Return) = k.button {
                    if let SceneEnum::WorldGen(scene) = app.scene {
                        let mut world = scene.into_world();

                        let load_save_manager = LoadSaveManager::new();
                        // TODO(ROO4JcDl): Unwrap
                        load_save_manager.save_world(&world).unwrap();

                        let (creature_id, pos) = world.create_scenario().expect("No playable scenario found");
                        world.dump_events("lore.log", &app.context.resources);

                        let creature = world.creatures.get(&creature_id);
                        let species = app.context.resources.species.get(&creature.species);
                        let mut player = Actor::from_creature(Coord2::xy(16, 16), AiGroups::player(), creature_id, &creature, &creature.species, &species, &world, &app.context.resources);
                        drop(creature);

                        let mut rng = Rng::seeded(creature_id).derive("equipment");
                        let _ = player.inventory.add(ItemFactory::starter_weapon(&mut rng, &app.context.resources).make());

                        player.inventory.auto_equip(&app.context.resources);

                        let chunk = GameState::from_world_tile(&world, &app.context.resources, ChunkCoord::new(pos, ChunkLayer::Surface), player);
                        let mut scene = GameSceneState::new(world, chunk);
                        scene.init(&mut app.context);
                        app.scene = SceneEnum::Game(scene);

                        continue
                    }
                }

                if let Button::Keyboard(Key::F4) = k.button {
                    if let SceneEnum::Game(scene) = app.scene {
                        let chunk = GameState::playground(&app.context.resources, scene.state.player().clone(), &scene.world);
                        let mut scene = GameSceneState::new(scene.world, chunk);
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

        let events: Vec<BusEvent> = app.context.event_bus.drain(..).collect();
        for event in events {
            app.event(&event);

            match event {
                BusEvent::QuitToMenu => {
                    app.scene = SceneEnum::MainMenu(MainMenuScene::new());
                },
                BusEvent::CreateNewCharacter => {
                    if let SceneEnum::Game(state) = app.scene {
                        app.scene = SceneEnum::WorldGen(WorldGenScene::continue_simulation(state.world, &app.context.resources));
                    }
                },
                _ => ()
            }
        }

    }
}
