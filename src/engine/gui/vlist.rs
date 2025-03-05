use crate::GameContext;

use super::{container::{Container, InnerContainer}, GUINode, Position};

pub struct VList {
    position: Position,
    inner: InnerContainer,
    gap: f64
}

impl VList {
    
    pub fn new(position: Position) -> VList {
        VList {
            position,
            inner: InnerContainer::new(),
            gap: 4.
        }
    }

}

impl Container for VList {
    
    fn container(&self) -> &InnerContainer {
        &self.inner
    }

    fn container_mut(&mut self) -> &mut InnerContainer {
        &mut self.inner
    }

    fn render_children(&mut self, ctx: &mut crate::engine::render::RenderContext, game_ctx: &mut GameContext, my_rect: [f64; 4]) {
        let layout_rect = ctx.layout_rect;
        ctx.layout_rect = my_rect;
        let gap = self.gap;
        for child in self.container_mut().children.iter_mut() {
            if let Some(gui_node) = Self::to_gui_node(child) {
                gui_node.render(ctx, game_ctx);
                ctx.layout_rect[1] += gui_node.min_size(ctx)[1] + gap;
            }
        }
        ctx.layout_rect = layout_rect;
    }
    
}

impl GUINode for VList {
    
    fn render(&mut self, ctx: &mut crate::engine::render::RenderContext, game_ctx: &mut GameContext) {
        let size = [600., 300.];
        let position = self.compute_position(&self.position, self.parent_rect(ctx), size);
        let rect = [position[0], position[1], size[0], size[1]];
        self.render_children(ctx, game_ctx, rect);
    }

    fn update(&mut self, update: &crate::engine::scene::Update, ctx: &mut GameContext) {
        self.update_children(update, ctx);
    }

}