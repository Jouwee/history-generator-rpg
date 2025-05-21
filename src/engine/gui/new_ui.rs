use graphics::{image, Transformed};
use ::image::ImageReader;
use piston::MouseButton;

use crate::{engine::{asset::image::ImageAsset, input::InputEvent, spritesheet::Spritesheet}, Color, GameContext, RenderContext};

#[derive(Debug)]
pub(crate) struct LayoutComponent {
    padding: [f64; 4],
    anchor: Anchor,
    anchor_margin: [f64; 4],
    size: [f64; 2],
    last_layout: [f64; 4],
}

impl LayoutComponent {

    pub(crate) fn new() -> Self {
        Self {
            padding: [0.; 4],
            anchor: Anchor::TopLeft,
            anchor_margin: [0.; 4],
            size: [24.; 2],
            last_layout: [0.; 4]
        }
    }

    pub(crate) fn compute_layout_rect(&mut self, ctx: &RenderContext) -> [f64; 4] {
        let size = self.size;
        let x = match &self.anchor {
            Anchor::TopLeft => ctx.layout_rect[0] + self.anchor_margin[0],
            Anchor::Center => ctx.layout_rect[0] / 2. + size[0] / 2. + self.anchor_margin[0],
            Anchor::TopRight => ctx.layout_rect[0] + ctx.layout_rect[2] - self.anchor_margin[2] - size[0],
        };
        let y = match &self.anchor {
            Anchor::TopLeft | Anchor::TopRight => ctx.layout_rect[1] + self.anchor_margin[1],
            Anchor::Center => ctx.layout_rect[1] / 2. + size[1] / 2. + self.anchor_margin[1],
        };
        self.last_layout = [x, y, size[0], size[1]];
        return self.last_layout;
    }

    pub(crate) fn compute_inner_layout_rect(&mut self, ctx: &RenderContext) -> [f64; 4] {
        let base_rect = self.compute_layout_rect(ctx);
        return [
            base_rect[0] + self.padding[0],
            base_rect[1] + self.padding[1],
            base_rect[2] - self.padding[2] - self.padding[0],
            base_rect[3] - self.padding[3] - self.padding[1],
        ];
    }

    pub(crate) fn padding(&mut self, padding: [f64; 4]) -> &mut Self {
        self.padding = padding;
        return self
    }

    pub(crate) fn size(&mut self, size: [f64; 2]) -> &mut Self {
        self.size = size;
        return self
    }

    pub(crate) fn anchor_top_right(&mut self, right: f64, top: f64) -> &mut Self {
        self.anchor = Anchor::TopRight;
        self.anchor_margin = [0., top, right, 0.];
        return self
    }

    pub(crate) fn anchor_center(&mut self) -> &mut Self {
        self.anchor = Anchor::Center;
        return self
    }

    pub(crate) fn hitbox(&self, cursor: &[f64; 2]) -> bool {
        let layout = &self.last_layout;
        return cursor[0] >= layout[0] && cursor[1] >= layout[1] && cursor[0] <= layout[0]+layout[2] && cursor[1] <= layout[1]+layout[3]
    }

}

// https://docs.godotengine.org/en/3.0/getting_started/step_by_step/ui_introduction_to_the_ui_system.html#how-to-change-the-anchor
#[derive(Debug)]
pub enum Anchor {
    TopLeft,
    // TopCenter,
    TopRight,
    // CenterLeft,
    Center,
    // CenterRight,
    // BottomLeft,
    // BottomCenter,
    // BottomRight,
}

// ---------------------

pub(crate) struct DialogWrapper<T> where T: UINode {
    value: Option<T>,
    close_button: Button,
}

impl<T, S> DialogWrapper<T> where T: UINode<State = S> {

    pub(crate) fn new() -> Self {
        let mut close_button = Button::text("Close");
        close_button.layout_component().anchor_top_right(0., 0.);
        Self {
            value: None,
            close_button
        }
    }

    pub(crate) fn show(&mut self, value: T) {
        self.value = Some(value)
    }

    pub(crate) fn hide(&mut self) -> Option<T> {
        return self.value.take();
    }

    pub(crate) fn render(&mut self, state: &mut S, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        if let Some(v) = &mut self.value {
            let rect = v.layout_component().compute_layout_rect(ctx);
            let position = [rect[0], rect[1]];
            let size = [rect[2], rect[3]];
            // TODO: Better spritesheets, and scaling
            let spritesheet = ImageReader::open("./assets/sprites/gui/dialog.png").unwrap().decode().unwrap();
            let spritesheet = Spritesheet::new(spritesheet, (24, 24));
            // Corners
            let transform = ctx.context.transform.trans(position[0], position[1]);
            image(spritesheet.sprite(0, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 24.);
            image(spritesheet.sprite(0, 2), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 24., position[1]);
            image(spritesheet.sprite(2, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 24., position[1] + size[1] - 24.);
            image(spritesheet.sprite(2, 2), transform, ctx.gl);
            // Borders
            let transform = ctx.context.transform.trans(position[0] + 24., position[1]).scale((size[0]-24.) / 24., 1.);
            image(spritesheet.sprite(1, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + 24., position[1] + size[1] - 24.).scale((size[0]-24.) / 24., 1.);
            image(spritesheet.sprite(1, 2), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0], position[1] + 24.).scale(1., (size[1]-24.) / 24.);
            image(spritesheet.sprite(0, 1), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 24., position[1] + 24.).scale(1., (size[1]-24.) / 24.);
            image(spritesheet.sprite(2, 1), transform, ctx.gl);
            // Body
            let transform = ctx.context.transform.trans(position[0] + 24., position[1] + 24.).scale((size[0]-24.) / 24., (size[1]-24.) / 24.);
            image(spritesheet.sprite(1, 1), transform, ctx.gl);

            v.render(state, ctx, game_ctx);

            let copy = ctx.layout_rect;
            ctx.layout_rect = v.layout_component().compute_layout_rect(ctx);
            self.close_button.render(&(), ctx, game_ctx);
            ctx.layout_rect = copy;

        }

    }

    pub(crate) fn input(&mut self, state: &mut S, evt: &crate::InputEvent, ctx: &mut GameContext) -> InputResult<()> {
        if let Some(value) = &mut self.value {
            match self.close_button.input(&mut (), evt, ctx) {
                InputResult::Consume(_) => {
                    self.hide();
                    return InputResult::Consume(());
                },
                _ => ()
            }
            if value.input(state, evt, ctx).is_consumed() {
                return InputResult::Consume(());
            }
        }
        InputResult::None
    }

}

// --------------------

pub(crate) trait UINode {
    type State;
    type Input;

    fn layout_component(&mut self) -> &mut LayoutComponent;

    fn init(&mut self, _state: &Self::State, _game_ctx: &mut GameContext) {}

    fn render(&mut self, _state: &Self::State, _ctx: &mut RenderContext, _game_ctx: &mut GameContext) {}

    fn input(&mut self, _state: &mut Self::State, _evt: &InputEvent, _ctx: &mut GameContext) -> InputResult<Self::Input> {
        return InputResult::None;
    }

}

pub(crate) enum InputResult<T> {
    None,
    Passthrough(T),
    Consume(T),
}

impl<T> InputResult<T> {
    pub(crate) fn is_consumed(&self) -> bool {
        match self {
            Self::Consume(_) => true,
            _ => false,
        }
    }
}

// --------------------

pub(crate) struct Button {
    layout: LayoutComponent, 
    text: String,
}

impl Button {

    pub(crate) fn text(text: &str) -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([48., 24.]);
        Self {
            layout,
            text: String::from(text)
        }
    }

}

impl UINode for Button {
    type State = ();
    type Input = ();

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let layout = self.layout.compute_inner_layout_rect(ctx);

        let frame = ImageReader::open("./assets/sprites/gui/button/frame.png").unwrap().decode().unwrap();
        let frame = Spritesheet::new(frame, (8, 8));

        let background = ImageAsset::new("gui/button/background.png");
        let background = game_ctx.assets.image(&background);

        let position = [layout[0], layout[1]];
        let size = [layout[2], layout[3]];
        // Background
        let transform = ctx.context.transform.trans(position[0], position[1]).scale(size[0] / 24., size[1] / 24.);
        image(&background.texture, transform, ctx.gl);
        // Corners
        let transform = ctx.context.transform.trans(position[0], position[1]);
        image(frame.sprite(0, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 8.);
        image(frame.sprite(0, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1]);
        image(frame.sprite(2, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + size[1] - 8.);
        image(frame.sprite(2, 2), transform, ctx.gl);
        // Borders
        let transform = ctx.context.transform.trans(position[0] + 8., position[1]).scale((size[0]-16.) / 8., 1.);
        image(frame.sprite(1, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + 8., position[1] + size[1] - 8.).scale((size[0]-16.) / 8., 1.);
        image(frame.sprite(1, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(frame.sprite(0, 1), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(frame.sprite(2, 1), transform, ctx.gl);

        ctx.text(&self.text, game_ctx.assets.font_standard(), [layout[0]as i32 + 4, layout[1] as i32 + 15], &Color::from_hex("ffffff"));
    }

    fn input(&mut self, _state: &mut Self::State, evt: &InputEvent, _game_ctx: &mut GameContext) -> InputResult<()> {
        match evt {
            InputEvent::Click { button: MouseButton::Left, pos } => {
                if self.layout.hitbox(pos) {
                    return InputResult::Consume(());
                }
            },
            _ => ()
        }
        return InputResult::None;
    }

}