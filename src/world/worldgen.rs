use std::{ops::ControlFlow, time::Instant};

use graphics::Transformed;
use image::ImageReader;
use opengl_graphics::{Filter, Texture, TextureSettings};

use crate::{engine::{assets::assets, audio::TrackMood, gui::UINode, input::InputEvent, render::RenderContext, scene::{Scene, Update}, COLOR_WHITE}, game::map_component::MapComponent, resources::resources::Resources, world::{date::Duration, site::SiteType}, GameContext};

use super::{history_generator::{WorldGenerationParameters, WorldHistoryGenerator}, world::World};

pub(crate) struct WorldGenScene {
    generator: WorldHistoryGenerator,
    map: MapComponent,
    banner_texture: Texture,
}

impl WorldGenScene {
    pub(crate) fn new(params: WorldGenerationParameters, resources: &Resources) -> WorldGenScene {
        let spritesheet = ImageReader::open("assets/sprites/banner.png").unwrap().decode().unwrap();
        let settings = TextureSettings::new().filter(Filter::Nearest);

        let mut scene = WorldGenScene {
            generator: WorldHistoryGenerator::seed_world(params.clone(), resources),
            map: MapComponent::new(),
            banner_texture: Texture::from_image(&spritesheet.to_rgba8(), &settings),
        };
        scene.build_tilemap();
        return scene
    }

    pub(crate) fn continue_simulation(mut world: World) -> WorldGenScene {
        world.generation_parameters.history_length = world.generation_parameters.history_length + 50;

        let spritesheet = ImageReader::open("assets/sprites/banner.png").unwrap().decode().unwrap();
        let settings = TextureSettings::new().filter(Filter::Nearest);

        let mut scene = WorldGenScene {
            generator: WorldHistoryGenerator::simulator(world),
            map: MapComponent::new(),
            banner_texture: Texture::from_image(&spritesheet.to_rgba8(), &settings),
        };
        scene.build_tilemap();
        return scene
    }

    pub(crate) fn build_tilemap(&mut self) {
        self.map.set_topology(&self.generator.world.map);
    }

    pub(crate) fn into_world(self) -> World {
        return self.generator.world
    }
}

impl Scene for WorldGenScene {
    type Input = ();

    fn init(&mut self, ctx: &mut GameContext) {
        ctx.audio.switch_music(TrackMood::Regular);
    }

    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        ctx.scale(2.);

        let copy = ctx.context.transform;
        ctx.context.transform = ctx.context.transform.trans(-16., -16.);
        self.map.render(&(), ctx, game_ctx);
        ctx.context.transform = copy;

        // Year banner
        let center = ctx.layout_rect[2] / 2.;
        let mut assets = assets();
        let font= assets.font_standard();
        ctx.texture(&self.banner_texture, ctx.at(center - 64., 0.));
        let text = self.generator.world.date_desc(&self.generator.world.date);
        let text_width = font.width(&text);
        ctx.text(&text, font, [(center - text_width / 2.).round() as i32, 16], &COLOR_WHITE);
        let text = "Press <enter> to start playing";
        let text_width = font.width(&text);
        ctx.text_shadow(&text, font, [(center - text_width / 2.).round() as i32, 40], &COLOR_WHITE);
    }

    fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {
        if self.generator.stop || self.generator.world.date.year() >= self.generator.parameters.history_length as i32 {
            return
        }
        let start = Instant::now();
        loop {
            // TODO(CF3fkX3): Too small and everything dies
            
            self.generator.simulate_step(Duration::months(3));
            // Simulate years until reach the max time per iteration, otherwise it takes longer than it needs
            if start.elapsed().as_secs_f64() >= _update.max_update_time {
                break;
            }
            if self.generator.stop || self.generator.world.date.year() >= self.generator.parameters.history_length as i32  {
                break;
            }
        }
        self.map.update_visible_sites(&self.generator.world, |_id, site| site.site_type == SiteType::Village);
    }

    fn input(&mut self, _evt: &InputEvent, _ctx: &mut GameContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }

    fn event(&mut self, _evt: &crate::engine::scene::BusEvent, _ctx: &mut GameContext) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}