use std::{hash::{DefaultHasher, Hash, Hasher}, ops::ControlFlow};

use graphics::{image, Transformed};
use ::image::ImageReader;
use piston::MouseButton;

use crate::{engine::{assets::assets, gui::{layout_component::LayoutComponent, tooltip::Tooltip, UIEvent, UINode}, spritesheet::Spritesheet, COLOR_WHITE}, GameContext, InputEvent, RenderContext};


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
            self.state_bitmask |= 0b0000_0010;
        } else {
            self.state_bitmask &= 0b1111_1101;
        }
    }

    pub(crate) fn is_selected(&self) -> bool {
        return self.state_bitmask & 0b0000_0010 > 0
    }

    pub(crate) fn set_hover(&mut self, selected: bool) {
        if selected {
            self.state_bitmask |= 0b0000_0001;
        } else {
            self.state_bitmask &= 0b1111_1110;
        }
    }

    pub(crate) fn is_hover(&self) -> bool {
        return self.state_bitmask & 0b0000_0001 > 0
    }

}

impl UINode for Button {
    type State = ();
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, _game_ctx: &mut GameContext) {
        let layout = self.layout.compute_inner_layout_rect(ctx.layout_rect);

        let background = assets().image(&self.background);

        let position = [layout[0], layout[1]];
        let size = [layout[2], layout[3]];
        // Background
        let transform = ctx.context.transform.trans(position[0], position[1]).scale(size[0] / 24., size[1] / 24.);
        image(&background.texture, transform, ctx.gl);

        let state_offset = match (self.is_selected(), self.is_hover()) {
            (false, false) => 0,
            (false, true) => 3,
            (true, _) => 6
        };

        // Corners
        let transform = ctx.context.transform.trans(position[0], position[1]);
        image(self.frame.sprite(state_offset + 0, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 8.);
        image(self.frame.sprite(state_offset + 0, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1]);
        image(self.frame.sprite(state_offset + 2, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + size[1] - 8.);
        image(self.frame.sprite(state_offset + 2, 2), transform, ctx.gl);
        // Borders
        let transform = ctx.context.transform.trans(position[0] + 8., position[1]).scale((size[0]-16.) / 8., 1.);
        image(self.frame.sprite(state_offset + 1, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + 8., position[1] + size[1] - 8.).scale((size[0]-16.) / 8., 1.);
        image(self.frame.sprite(state_offset + 1, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(self.frame.sprite(state_offset + 0, 1), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(self.frame.sprite(state_offset + 2, 1), transform, ctx.gl);

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
            InputEvent::Click { button: MouseButton::Left, pos } => {
                if self.layout.hitbox(pos) {
                    return ControlFlow::Break(UIEvent::ButtonClicked(self.key.as_ref().unwrap_or(&self.text).clone()));
                }
            },
            InputEvent::MouseMove { pos } => {
                let hit = self.layout.hitbox(pos);
                self.set_hover(hit);
                if let Some((hash, tooltip)) = &self.tooltip {
                    if hit {
                        ctx.tooltips.show_delayed_prehash(*hash, &tooltip, *pos);
                    } else {
                        ctx.tooltips.hide_prehash(*hash);
                    }
                }
            }
            _ => ()
        }
        return ControlFlow::Continue(());
    }

}