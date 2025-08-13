use std::ops::ControlFlow;

use graphics::{image, Transformed};
use ::image::ImageReader;

use crate::{engine::{gui::{button::Button, UIEvent, UINode}, spritesheet::Spritesheet}, GameContext, RenderContext};


pub(crate) struct DialogWrapper<T> where T: UINode {
    value: Option<T>,
    close_button: Option<Button>,
}

impl<T, S> DialogWrapper<T> where T: UINode<State = S, Input = UIEvent> {

    pub(crate) fn new() -> Self {
        let mut close_button = Button::text("X");
        close_button.layout_component().anchor_top_right(1., 1.).size([16., 16.]);
        Self {
            value: None,
            close_button: Some(close_button)
        }
    }

    pub(crate) fn hide_close_button(mut self) -> Self {
        self.close_button = None;
        return self;
    }

    pub(crate) fn show(&mut self, mut value: T, state: &S, game_ctx: &mut GameContext) {
        value.init(state, game_ctx);
        self.value = Some(value)
    }

    pub(crate) fn hide(&mut self, state: &mut S, game_ctx: &mut GameContext) -> Option<T> {
        if let Some(value) = &mut self.value {
            value.destroy(state, game_ctx)
        }
        return self.value.take();
    }

    pub(crate) fn is_visible(&self) -> bool {
        return self.value.is_some();
    }

    pub(crate) fn render(&mut self, state: &mut S, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        if let Some(v) = &mut self.value {
            let rect = v.layout_component().compute_layout_rect(ctx.layout_rect);
            let position = [rect[0], rect[1]];
            let size = [rect[2], rect[3]];
            // TODO: Better spritesheets, and scaling
            let spritesheet = ImageReader::open("./assets/sprites/gui/dialog.png").unwrap().decode().unwrap();
            let spritesheet = Spritesheet::new(spritesheet, (24, 24));
            // Body
            let transform = ctx.context.transform.trans(position[0] + 24., position[1] + 24.).scale((size[0]-24.) / 24., (size[1]-24.) / 24.);
            image(spritesheet.sprite(1, 1), transform, ctx.gl);
            // Borders
            let transform = ctx.context.transform.trans(position[0] + 24., position[1]).scale((size[0]-24.) / 24., 1.);
            image(spritesheet.sprite(1, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + 24., position[1] + size[1] - 24.).scale((size[0]-24.) / 24., 1.);
            image(spritesheet.sprite(1, 2), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0], position[1] + 24.).scale(1., (size[1]-24.) / 24.);
            image(spritesheet.sprite(0, 1), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 24., position[1] + 24.).scale(1., (size[1]-24.) / 24.);
            image(spritesheet.sprite(2, 1), transform, ctx.gl);
            // Corners
            let transform = ctx.context.transform.trans(position[0], position[1]);
            image(spritesheet.sprite(0, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 24.);
            image(spritesheet.sprite(0, 2), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 24., position[1]);
            image(spritesheet.sprite(2, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 24., position[1] + size[1] - 24.);
            image(spritesheet.sprite(2, 2), transform, ctx.gl);

            v.render(state, ctx, game_ctx);

            if let Some(close_button) = &mut self.close_button {
                let copy = ctx.layout_rect;
                ctx.layout_rect = v.layout_component().compute_layout_rect(ctx.layout_rect);
                close_button.render(&(), ctx, game_ctx);
                ctx.layout_rect = copy;
            }

        }

    }

    pub(crate) fn input(&mut self, state: &mut S, evt: &crate::InputEvent, ctx: &mut GameContext) -> ControlFlow<()> {
        if let Some(value) = &mut self.value {
            if let Some(close_button) = &mut self.close_button {
                match close_button.input(&mut (), evt, ctx) {
                    ControlFlow::Break(_) => {
                        self.hide(state, ctx);
                        return ControlFlow::Break(());
                    },
                    _ => ()
                }
            }
            match value.input(state, evt, ctx) {
                ControlFlow::Break(UIEvent::DialogClosed) => {
                    self.hide(state, ctx);
                    return ControlFlow::Break(());
                },
                _ => return ControlFlow::Break(())
            }
        }
        ControlFlow::Continue(())
    }

}