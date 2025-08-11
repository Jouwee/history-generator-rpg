use crate::{engine::{assets::assets, geometry::{Coord2, Size2D, Vec2}, gui::{button::Button, UINode}, input::InputEvent, render::RenderContext, scene::Update, COLOR_WHITE}, game::map_component::MapComponent, world::{world::World}, GameContext};
use piston::{Button as Btn, ButtonState, Key, MouseButton};

use super::InputEvent as OldInputEvent;

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
        self.map.update_visible_units(world, |id, _unit| world.codex.unit(id).is_some());

        self.offset = Vec2::xy(player_pos.x as f32 * 16., player_pos.y as f32 * 16.);
        self.player_pos = player_pos.clone();

        self.world_size = world.map.size;

    }

    pub(crate) fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        ctx.push();
        ctx.center_camera_on([self.offset.x.max(0.) as f64, self.offset.y.max(0.) as f64]);
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
        for (coord, _, _) in self.map.names.iter() {
            if self.mouse_over == *coord {
                let text = "Click to fast-travel";
                let width = font.width(text);
                ctx.text_shadow(text, font, [coord.x * 16 - (width / 2.) as i32 + 8, coord.y * 16], &COLOR_WHITE);
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

    pub(crate) fn input(&mut self, evt: &OldInputEvent, ctx: &mut GameContext) -> MapModalEvent {
        if evt.button_args.state == ButtonState::Press {
            match evt.button_args.button {
                Btn::Keyboard(Key::M) | Btn::Keyboard(Key::Escape) => {
                    return MapModalEvent::Close
                }
                _ => ()
            }
        }
        if self.close_button.input(&mut (), &evt.evt, ctx).is_break() {
            return MapModalEvent::Close;
        }
        let camera = ctx.display_context.camera_rect;
        let clamp = [
            [camera[2] / 2. + 16., (self.world_size.0 as f64 * 16.) - camera[2] / 2.],
            [camera[3] / 2. + 16., (self.world_size.1 as f64 * 16.) - camera[3] / 2.],
        ];
        match evt.evt {
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
                        return MapModalEvent::InstaTravelTo(mouse)
                    }
                }
            }
            _ => ()
        }
        return MapModalEvent::None
    }

}

pub(crate) enum MapModalEvent {
    None,
    Close,
    InstaTravelTo(Coord2)
}