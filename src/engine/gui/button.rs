use std::{hash::{DefaultHasher, Hash, Hasher}, ops::ControlFlow};

use graphics::{Transformed, Image as GlImage};
use ::image::ImageReader;
use piston::MouseButton;

use crate::{commons::bitmask::{bitmask_get, bitmask_set, bitmask_unset}, engine::{assets::assets, audio::SoundEffect, gui::{layout_component::LayoutComponent, tooltip::Tooltip, UIEvent, UINode}, spritesheet::Spritesheet, COLOR_WHITE}, GameContext, InputEvent, RenderContext};

const STATE_HOVER: u8 = 0b0000_0001;
const STATE_PRESSED: u8 = 0b0000_0010;
const STATE_SELECTED: u8 = 0b0000_0100;

pub(crate) struct Button {
    layout: LayoutComponent,
    text: String,
    key: Option<String>,
    background: String,
    frame: Spritesheet,
    tooltip: Option<(u64, Tooltip)>,
    state_bitmask: u8,
}

impl Button {

    pub(crate) fn text(text: &str) -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([24., 24.]);
        let frame = ImageReader::open("./assets/sprites/gui/button/frame.png").unwrap().decode().unwrap();
        let frame = Spritesheet::new(frame, (8, 8));

        Self {
            layout,
            text: String::from(text),
            key: None,
            background: String::from("gui/button/background.png"),
            tooltip: None,
            frame,
            state_bitmask: 0,
        }
    }

    pub(crate) fn image(image: String) -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([24., 24.]);

        let frame = ImageReader::open("./assets/sprites/gui/button/frame.png").unwrap().decode().unwrap();
        let frame = Spritesheet::new(frame, (8, 8));

        Self {
            layout,
            text: String::from(""),
            key: None,
            background: image,
            tooltip: None,
            frame,
            state_bitmask: 0,
        }
    }

    pub(crate) fn key(mut self, key: &str) -> Self {
        self.key = Some(String::from(key));
        return self;
    }

    pub(crate) fn tooltip(mut self, tooltip: Tooltip) -> Self {
        let mut hasher = DefaultHasher::new();
        tooltip.hash(&mut hasher);
        let hash = hasher.finish();
        self.tooltip = Some((hash, tooltip));
        return self
    }

    pub(crate) fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }

    pub(crate) fn set_selected(&mut self, selected: bool) {
        if selected {
            self.state_bitmask = bitmask_set(self.state_bitmask, STATE_SELECTED);
        } else {
            self.state_bitmask = bitmask_unset(self.state_bitmask, STATE_SELECTED);
        }
    }

    pub(crate) fn is_selected(&self) -> bool {
        bitmask_get(self.state_bitmask, STATE_SELECTED)
    }

    pub(crate) fn set_hover(&mut self, hover: bool) {
        if hover {
            self.state_bitmask = bitmask_set(self.state_bitmask, STATE_HOVER);
        } else {
            self.state_bitmask = bitmask_unset(self.state_bitmask, STATE_HOVER);
        }
    }

    pub(crate) fn is_hover(&self) -> bool {
        bitmask_get(self.state_bitmask, STATE_HOVER)
    }

    pub(crate) fn set_pressed(&mut self, pressed: bool) {
        if pressed {
            self.state_bitmask = bitmask_set(self.state_bitmask, STATE_PRESSED);
        } else {
            self.state_bitmask = bitmask_unset(self.state_bitmask, STATE_PRESSED);
        }
    }

    pub(crate) fn is_pressed(&self) -> bool {
        bitmask_get(self.state_bitmask, STATE_PRESSED)
    }


}

impl UINode for Button {
    type State = ();
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, _game_ctx: &mut GameContext) {
        let draw_state = ctx.context.draw_state;
        let image = GlImage::new();

        let layout = self.layout.compute_inner_layout_rect(ctx.layout_rect);

        let background = assets().image(&self.background);

        let position = [layout[0], layout[1]];
        let size = [layout[2], layout[3]];
        // Background
        let transform = ctx.context.transform.trans(position[0], position[1]).scale(size[0] / 24., size[1] / 24.);
        image.draw(&background.texture, &draw_state, transform, ctx.gl);

        let state_offset = match (self.is_selected(), self.is_pressed(), self.is_hover()) {
            (true, _, _) => 6,
            (false, true, _) => 9,
            (false, false, true) => 3,
            (false, false, false) => 0,
        };

        // Corners
        let transform = ctx.context.transform.trans(position[0], position[1]);
        image.draw(self.frame.sprite(state_offset + 0, 0), &draw_state, transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 8.);
        image.draw(self.frame.sprite(state_offset + 0, 2), &draw_state, transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1]);
        image.draw(self.frame.sprite(state_offset + 2, 0), &draw_state, transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + size[1] - 8.);
        image.draw(self.frame.sprite(state_offset + 2, 2), &draw_state, transform, ctx.gl);
        // Borders
        let transform = ctx.context.transform.trans(position[0] + 8., position[1]).scale((size[0]-16.) / 8., 1.);
        image.draw(self.frame.sprite(state_offset + 1, 0), &draw_state, transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + 8., position[1] + size[1] - 8.).scale((size[0]-16.) / 8., 1.);
        image.draw(self.frame.sprite(state_offset + 1, 2), &draw_state, transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image.draw(self.frame.sprite(state_offset + 0, 1), &draw_state, transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image.draw(self.frame.sprite(state_offset + 2, 1), &draw_state, transform, ctx.gl);

        let text_width = assets().font_standard().width(&self.text);
        let text_height = assets().font_standard().line_height();
        let pos = [
            layout[0] + (layout[2] / 2.) - (text_width / 2.),
            layout[1] + (layout[3] / 2.) - (text_height / 2.) + text_height -1.,
        ];
        ctx.text(&self.text, assets().font_standard(), [pos[0] as i32, pos[1] as i32], &COLOR_WHITE);
    }

    fn input(&mut self, _state: &mut Self::State, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<UIEvent> {
        match evt {
            InputEvent::MousePress { button: MouseButton::Left, pos } => {
                if self.layout.hitbox(pos) {
                    self.set_pressed(true);
                } else {
                    self.set_pressed(false);
                }
            }
            InputEvent::Click { button: MouseButton::Left, pos } => {
                if self.layout.hitbox(pos) {
                    ctx.audio.play_once(SoundEffect::new(vec!("ui/button-click.mp3")));
                    self.set_pressed(false);
                    return ControlFlow::Break(UIEvent::ButtonClicked(self.key.as_ref().unwrap_or(&self.text).clone()));
                }
            },
            InputEvent::MouseMove { pos } => {
                let hit = self.layout.hitbox(pos);
                if !hit {
                    self.set_pressed(false);
                }
                self.set_hover(hit);
                if let Some((hash, tooltip)) = &self.tooltip {
                    if hit {
                        ctx.tooltips.show_delayed_prehash(*hash, &tooltip, *pos);
                    } else {
                        ctx.tooltips.hide_prehash(*hash);
                    }
                } else {
                    ctx.tooltips.hide();
                }
            }
            _ => ()
        }
        return ControlFlow::Continue(());
    }

}