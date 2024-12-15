use crate::engine::{render::RenderContext, Color};

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
    fn render(&mut self, ctx: &mut RenderContext) {
        let size = [600., 400.];
        let position = self.compute_position(&self.position, self.parent_rect(ctx), size);
        let rect = [position[0], position[1], size[0], size[1]];
        ctx.rectangle_fill([position[0], position[1], size[0], size[1]], Color::from_hex("1a273cEE"));
        self.render_children(ctx, rect);
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