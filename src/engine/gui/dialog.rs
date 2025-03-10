use graphics::{image, Transformed};
use ::image::ImageReader;

use crate::{engine::{render::RenderContext, spritesheet::Spritesheet}, GameContext};

use super::{container::{Container, InnerContainer}, GUINode, Position};

pub struct Dialog {
    position: Position,
    inner: InnerContainer
}

impl Dialog {
    pub fn new() -> Dialog {
        Dialog { position: Position::Centered, inner: InnerContainer::new() }
    }
}

impl GUINode for Dialog {
    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let size = [600., 300.];
        let position = self.compute_position(&self.position, self.parent_rect(ctx), size);
        let rect = [position[0], position[1], size[0], size[1]];
        // TODO: Better spritesheets, and scaling
        let spritesheet = ImageReader::open("./assets/sprites/box.png").unwrap().decode().unwrap();
        let spritesheet = Spritesheet::new(spritesheet, (8, 8));
        // Corners
        let transform = ctx.context.transform.trans(position[0], position[1]);
        image(spritesheet.sprite(0, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 8.);
        image(spritesheet.sprite(0, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1]);
        image(spritesheet.sprite(2, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + size[1] - 8.);
        image(spritesheet.sprite(2, 2), transform, ctx.gl);
        // Borders
        let transform = ctx.context.transform.trans(position[0] + 8., position[1]).scale((size[0]-16.) / 8., 1.);
        image(spritesheet.sprite(1, 0), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + 8., position[1] + size[1] - 8.).scale((size[0]-16.) / 8., 1.);
        image(spritesheet.sprite(1, 2), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0], position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(spritesheet.sprite(0, 1), transform, ctx.gl);
        let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + 8.).scale(1., (size[1]-16.) / 8.);
        image(spritesheet.sprite(2, 1), transform, ctx.gl);
        // Body
        let transform = ctx.context.transform.trans(position[0] + 8., position[1] + 8.).scale((size[0]-16.) / 8., (size[1]-16.) / 8.);
        image(spritesheet.sprite(1, 1), transform, ctx.gl);
        self.render_children(ctx, game_ctx, rect);
    }
}

impl Container for Dialog {

    fn container(&self) -> &InnerContainer {
        &self.inner
    }

    fn container_mut(&mut self) -> &mut InnerContainer {
        &mut self.inner
    }

}

pub enum ButtonEvent {
    None,
    Click
}