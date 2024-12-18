use super::{container::{Container, InnerContainer}, GUINode, Position};

pub struct VList {
    position: Position,
    inner: InnerContainer
}

impl VList {
    
    pub fn new(position: Position) -> VList {
        VList {
            position,
            inner: InnerContainer::new()
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

    fn render_children(&mut self, ctx: &mut crate::engine::render::RenderContext, my_rect: [f64; 4]) {
        let layout_rect = ctx.layout_rect;
        ctx.layout_rect = my_rect;
        for child in self.container_mut().children.values_mut() {
            if let Some(gui_node) = Self::to_gui_node(child) {
                gui_node.render(ctx);
                ctx.layout_rect[1] += gui_node.min_size(ctx)[1];
            }
        }
        ctx.layout_rect = layout_rect;
    }
    
}

impl GUINode for VList {
    
    fn render(&mut self, ctx: &mut crate::engine::render::RenderContext) {
        let size = [600., 400.];
        let position = self.compute_position(&self.position, self.parent_rect(ctx), size);
        let rect = [position[0], position[1], size[0], size[1]];
        self.render_children(ctx, rect);
    }

}