use crate::GameContext;

use super::{container::{Container, InnerContainer}, GUINode, Position};

pub struct HList {
    position: Position,
    inner: InnerContainer,
    gap: f64,
    pub size: Option<[f64; 2]>
}

impl HList {
    
    pub fn new(position: Position) -> HList {
        HList {
            position,
            inner: InnerContainer::new(),
            gap: 4.,
            size: None,
        }
    }

}

impl Container for HList {
    
    fn container(&self) -> &InnerContainer {
        &self.inner
    }

    fn container_mut(&mut self) -> &mut InnerContainer {
        &mut self.inner
    }

    fn render_children(&mut self, ctx: &mut crate::engine::render::RenderContext, game_ctx: &GameContext, my_rect: [f64; 4]) {
        let layout_rect = ctx.layout_rect;
        ctx.layout_rect = my_rect;
        let gap = self.gap;
        for child in self.container_mut().children.iter_mut() {
            if let Some(gui_node) = Self::to_gui_node(child) {
                gui_node.render(ctx, game_ctx);
                ctx.layout_rect[0] += gui_node.min_size(ctx)[0] + gap;
            }
        }
        ctx.layout_rect = layout_rect;
    }
    
}

impl GUINode for HList {
    
    fn render(&mut self, ctx: &mut crate::engine::render::RenderContext, game_ctx: &GameContext) {
        let size = match self.size {
            Some(size) => size,
            None => [400., 100.]
        };
        let position = self.compute_position(&self.position, self.parent_rect(ctx), size);
        let rect = [position[0], position[1], size[0], size[1]];
        self.render_children(ctx, game_ctx, rect);
    }

    fn update(&mut self, update: &crate::engine::scene::Update, ctx: &mut GameContext) {
        self.update_children(update, ctx);
    }

}