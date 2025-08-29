use std::{ops::ControlFlow, sync::Arc};

use crate::{engine::{assets::ImageSheet, gui::{layout_component::LayoutComponent, UIEvent, UINode}, input::InputEvent}, GameContext, RenderContext};

/// A container
pub(crate) struct SimpleContainer {
    layout: LayoutComponent,
    auto_layout: Box<dyn AutoLayout>,
    background: Option<Arc<ImageSheet>>,
    max_scroll: f64,
    scroll: f64,
    children: Vec<Box<dyn UINode<State = (), Input = UIEvent>>>
}

impl SimpleContainer {

    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([24., 24.]);

        Self {
            layout,
            auto_layout: Box::new(VerticalAutoLayout::new()),
            background: None,
            scroll: 0.,
            max_scroll: 0.,
            children: Vec::new()
        }
    }

    pub(crate) fn background(mut self, background: Arc<ImageSheet>) -> Self {
        self.background = Some(background);
        return self;
    }

    pub(crate) fn clear(&mut self) {
        self.children.clear();
    }

    pub(crate) fn add<C>(&mut self, child: C) where C: UINode<State = (), Input = UIEvent> + 'static {
        self.children.push(Box::new(child));
    }

}

impl UINode for SimpleContainer {
    type State = ();
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let copy = ctx.layout_rect;

        if let Some(background) = &self.background {
            let layout = self.layout.compute_layout_rect(ctx.layout_rect);
            background.draw_as_scalable(layout, ctx);
        }

        let layout = self.layout.compute_inner_layout_rect(ctx.layout_rect);
        
        // SMELL: *2 because this is screen space, doesn't share the context
        let mut clip_rect = [
            layout[0] as u32 * 2,
            layout[1] as u32 * 2,
            layout[2] as u32 * 2,
            layout[3] as u32 * 2,
        ];
        if let Some(rect) = ctx.context.draw_state.scissor {
            if let Some(intersection) = intersection(clip_rect, rect) {
                clip_rect = intersection
            } else {
                // Invisible
                return
            }

        }
        let old_clip = ctx.set_clip_rect(Some(clip_rect));

        self.auto_layout.reset_layout(layout);
        for child in self.children.iter_mut() {
            child.recompute_layout(layout, game_ctx);
            ctx.layout_rect = self.auto_layout.layout_child(layout, child.layout_component());
            ctx.layout_rect[1] -= self.scroll;
            child.render(&(), ctx, game_ctx);
        }

        self.max_scroll = ((self.auto_layout.current_size()[1] - layout[1]) - layout[3]).max(0.);

        ctx.layout_rect = copy;
        ctx.set_clip_rect(old_clip);
    }

    fn input(&mut self, _state: &mut Self::State, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        if let InputEvent::Scroll { pos, offset } = evt {
            if self.max_scroll > 0. && self.layout.hitbox(pos) {
                self.scroll = (self.scroll - offset * 8.).clamp(0., self.max_scroll);
                return ControlFlow::Break(UIEvent::None);
            }
        }
        for child in self.children.iter_mut() {
            child.input(&mut (), evt, ctx)?;
        };
        return ControlFlow::Continue(())
    }

}

pub(crate) trait AutoLayout {

    fn reset_layout(&mut self, layout_rect: [f64; 4]);

    fn layout_child(&mut self, layout_rect: [f64; 4], layout: &mut LayoutComponent) -> [f64; 4];

    fn current_size(&mut self) -> [f64; 2];

}

pub(crate) struct VerticalAutoLayout {
    gap: f64,
    rect: [f64; 4]
}

impl VerticalAutoLayout {

    pub(crate) fn new() -> Self {
        Self {
            gap: 4.,
            rect: [0.; 4]
        }
    }

}

impl AutoLayout for VerticalAutoLayout {

    fn reset_layout(&mut self, layout_rect: [f64; 4]) {
        self.rect = layout_rect;
    }

    fn layout_child(&mut self, layout_rect: [f64; 4], child_layout: &mut LayoutComponent) -> [f64; 4] {
        let child = child_layout.compute_layout_rect(layout_rect);
        self.rect[3] = child[3];
        let copy = self.rect.clone();
        self.rect[1] += child[3] + self.gap;
        return copy;
    }

    fn current_size(&mut self) -> [f64; 2] {
        return [self.rect[0], self.rect[1] - self.gap];
    }

}

fn intersection(rect_a: [u32; 4], rect_b: [u32; 4]) -> Option<[u32; 4]> {
    let x1 = rect_a[0];
    let x2 = rect_a[0] + rect_a[2];
    let y1 = rect_a[1];
    let y2 = rect_a[1] + rect_a[2];

    let x3 = rect_b[0];
    let x4 = rect_b[0] + rect_b[2];
    let y3 = rect_b[1];
    let y4 = rect_b[1] + rect_b[2];

    // Intersect
    let x5 = x1.max(x3);
    let y5 = y1.max(y3);
    let x6 = x2.min(x4);
    let y6 = y2.min(y4);

    if x5 < x6 && y5 < y6 {
        return Some([x5, y5, (x6 - x5), (y6 - y5)])
    } else {
        return None
    }   
}