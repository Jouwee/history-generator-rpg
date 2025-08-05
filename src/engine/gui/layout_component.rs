use crate::engine::render::RenderContext;

#[derive(Debug)]
pub(crate) struct LayoutComponent {
    padding: [f64; 4],
    anchor: Anchor,
    anchor_margin: [f64; 4],
    size: [f64; 2],
    pub(crate) last_layout: [f64; 4],
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

    pub(crate) fn compute_layout_rect(&mut self, layout_rect: [f64; 4]) -> [f64; 4] {
        let size = self.size;
        let x = match &self.anchor {
            Anchor::TopLeft | Anchor::BottomLeft => layout_rect[0] + self.anchor_margin[0],
            Anchor::Center | Anchor::BottomCenter => layout_rect[0] + (layout_rect[2] / 2.) - (size[0] / 2.) + self.anchor_margin[0],
            Anchor::TopRight => layout_rect[0] + layout_rect[2] - self.anchor_margin[2] - size[0],
        };
        let y = match &self.anchor {
            Anchor::TopLeft | Anchor::TopRight => layout_rect[1] + self.anchor_margin[1],
            Anchor::Center => layout_rect[1] + (layout_rect[3] / 2.) - (size[1] / 2.) + self.anchor_margin[1],
            Anchor::BottomLeft | Anchor::BottomCenter => layout_rect[1] + layout_rect[3] - size[1] + self.anchor_margin[3],
        };
        self.last_layout = [x, y, size[0], size[1]];
        return self.last_layout;
    }

    pub(crate) fn compute_inner_layout_rect(&mut self, layout_rect: [f64; 4]) -> [f64; 4] {
        let base_rect = self.compute_layout_rect(layout_rect);
        return [
            base_rect[0] + self.padding[0],
            base_rect[1] + self.padding[1],
            base_rect[2] - self.padding[2] - self.padding[0],
            base_rect[3] - self.padding[3] - self.padding[1],
        ];
    }

    pub(crate) fn on_layout<F>(&mut self, mut callback: F, ctx: &mut RenderContext) where F: FnMut(&mut RenderContext) -> () {
        let copy = ctx.layout_rect;
        ctx.layout_rect = self.compute_inner_layout_rect(ctx.layout_rect);
        callback(ctx);
        ctx.layout_rect = copy;
    }

    pub(crate) fn padding(&mut self, padding: [f64; 4]) -> &mut Self {
        self.padding = padding;
        return self
    }

    pub(crate) fn size(&mut self, size: [f64; 2]) -> &mut Self {
        self.size = size;
        return self
    }

    pub(crate) fn anchor_top_left(&mut self, left: f64, top: f64) -> &mut Self {
        self.anchor = Anchor::TopLeft;
        self.anchor_margin = [left, top, 0., 0.];
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

    pub(crate) fn anchor_bottom_left(&mut self, left: f64, bottom: f64) -> &mut Self {
        self.anchor = Anchor::BottomLeft;
        self.anchor_margin = [left, 0., 0., bottom];
        return self
    }

    pub(crate) fn anchor_bottom_center(&mut self, center: f64, bottom: f64) -> &mut Self {
        self.anchor = Anchor::BottomCenter;
        self.anchor_margin = [center, 0., 0., bottom];
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
    BottomLeft,
    BottomCenter,
    // BottomRight,
}