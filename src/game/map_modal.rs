use std::ops::ControlFlow;

use crate::{engine::{assets::assets, geometry::{Coord2, Size2D, Vec2}, gui::{button::Button, UINode}, input::InputEvent, render::RenderContext, scene::Update, Color, COLOR_WHITE}, game::map_component::MapComponent, loc, world::world::World, GameContext};
use piston::{Key, MouseButton};

pub(crate) struct MapModal {
    world_size: Size2D,
    map: MapComponent,
    offset: Vec2,
    player_pos: Coord2,
    mouse_over: Coord2,
    close_button: Button
}

impl MapModal {

    pub(crate) fn new() -> MapModal {
        let mut close_button = Button::text("Close");
        close_button.layout_component().anchor_top_right(0., 0.).size([32., 20.]);

        MapModal {
            map: MapComponent::new(),
            offset: Vec2::xy(128.*16., 128.*16.),
            player_pos: Coord2::xy(0, 0),
            mouse_over: Coord2::xy(0, 0),
            world_size: Size2D(0, 0),
            close_button
        }
    }

    pub(crate) fn init(&mut self, world: &World, player_pos: &Coord2) {
        self.map.set_topology(&world.map);
        // TODO:
        self.map.update_visible_sites(world, |id, _site| world.codex.site(id).is_some());

        self.offset = Vec2::xy(player_pos.x as f32 * 16., player_pos.y as f32 * 16.);
        self.player_pos = player_pos.clone();

        self.world_size = world.map.size;

    }

    pub(crate) fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        ctx.rectangle_fill(ctx.layout_rect, &Color::from_hex("09071480"));
        ctx.push();

        let clamp = self.get_offset_clamp(ctx.camera_rect);
        ctx.center_camera_on([
            self.offset.x.clamp(clamp[0][0] as f32, clamp[0][1] as f32) as f64,
            self.offset.y.clamp(clamp[1][0] as f32, clamp[1][1] as f32) as f64
        ]);

        self.map.render(&(), ctx, game_ctx);

        let cursor = [self.player_pos.x * 16, self.player_pos.y * 16];
        
        let cursor_clamp = [
            cursor[0].clamp(ctx.camera_rect[0] as i32, ctx.camera_rect[0] as i32 + ctx.camera_rect[2] as i32 - 16),
            cursor[1].clamp(ctx.camera_rect[1] as i32, ctx.camera_rect[1] as i32 + ctx.camera_rect[3] as i32 - 16),
        ];
        if cursor != cursor_clamp {
            ctx.image(&"map_tiles/player_offscreen.png", cursor_clamp);
        } else {
            ctx.image(&"map_tiles/player.png", cursor_clamp);
        }

        let mut l_assets = assets();
        let font = l_assets.font_standard();
        for (coord, name, major) in self.map.names.iter() {
            if self.mouse_over == *coord {
                let text = loc!("map-modal-click-fast-travel");
                let width = font.width(text);
                ctx.text_shadow(text, font, [coord.x * 16 - (width / 2.) as i32 + 8, coord.y * 16 - 8], &COLOR_WHITE);

                if !major {
                    let text = name;
                    let width = font.width(text);
                    ctx.text_shadow(text, font, [coord.x * 16 - (width / 2.) as i32 + 8, coord.y * 16 + 20], &COLOR_WHITE);
                }
                break;
            }
        }
        drop(l_assets);

        let _ = ctx.try_pop();
        // Control
        ctx.image(&"controls/right_click.png", [ctx.layout_rect[2] as i32 - 88, ctx.layout_rect[3] as i32 - 24]);
        ctx.text("Drag to move", assets().font_standard(), [ctx.layout_rect[2] as i32 - 72, ctx.layout_rect[3] as i32 - 14], &COLOR_WHITE);
        self.close_button.render(&(), ctx, game_ctx);
    }

    pub(crate) fn update(&mut self, _update: &Update, _ctx: &mut GameContext) {}

    pub(crate) fn input(&mut self, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<MapModalEvent, ()> {
        if self.close_button.input(&mut (), &evt, ctx).is_break() {
            return ControlFlow::Break(MapModalEvent::Close)
        }
        let camera = ctx.display_context.camera_rect;
        let clamp = self.get_offset_clamp(camera);
        match evt {
            InputEvent::Key { key: Key::M } | InputEvent::Key { key: Key::Escape } => {
                return ControlFlow::Break(MapModalEvent::Close)
            }
            InputEvent::MouseMove { pos } => {
                self.mouse_over = Coord2::xy(
                    ((pos[0] + camera[0] as f64) / 16.) as i32, 
                    ((pos[1] + camera[1] as f64) / 16.) as i32
                );
            },
            InputEvent::Drag { button: MouseButton::Left, offset } => {
                self.offset.x = (self.offset.x - offset[0] as f32).clamp(clamp[0][0] as f32, clamp[0][1] as f32);
                self.offset.y = (self.offset.y - offset[1] as f32).clamp(clamp[1][0] as f32, clamp[1][1] as f32);
            },
            InputEvent::Click { button: MouseButton::Left, pos } => {
                let mouse = Coord2::xy(
                    ((pos[0] + camera[0] as f64) / 16.) as i32, 
                    ((pos[1] + camera[1] as f64) / 16.) as i32
                );
                for (coord, _, _) in self.map.names.iter() {
                    if mouse == *coord {
                        return ControlFlow::Break(MapModalEvent::InstaTravelTo(mouse))
                    }
                }
            }
            _ => ()
        }
        return ControlFlow::Break(MapModalEvent::None)
    }


    fn get_offset_clamp(&self, camera_rect: [f64; 4]) -> [[f64; 2]; 2] {
        let map_size = [self.world_size.0 as f64 * 16., self.world_size.1 as f64 * 16.];
        let x;
        if camera_rect[2] > map_size[0] {
            x = [map_size[0] / 2.; 2]
        } else {
            x = [camera_rect[2] / 2., map_size[0] - camera_rect[2] / 2.]
        }

        let y;
        if camera_rect[3] > map_size[1] {
            y = [map_size[1] / 2.; 2]
        } else {
            y = [camera_rect[3] / 2., map_size[1] - camera_rect[3] / 2.]
        }
        return [x, y]
    }

}

pub(crate) enum MapModalEvent {
    None,
    Close,
    InstaTravelTo(Coord2)
}