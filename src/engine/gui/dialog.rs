use graphics::{image, Transformed};
use ::image::ImageReader;

use crate::{engine::{gui::{button::Button, InputResult, UINode}, spritesheet::Spritesheet}, GameContext, RenderContext};


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