use std::ops::ControlFlow;

use crate::{engine::{gui::{layout_component::LayoutComponent, UIEvent, UINode}, Color}, GameContext, RenderContext};


/// A simple, layout-less container
pub(crate) struct SimpleContainer {
    layout: LayoutComponent,
    auto_layout: Box<dyn AutoLayout>,
    children: Vec<Box<dyn UINode<State = (), Input = UIEvent>>>
}

impl SimpleContainer {

    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([24., 24.]);

        Self {
            layout,
            auto_layout: Box::new(VerticalAutoLayout::new()),
            children: Vec::new()
        }
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
        let layout = self.layout.compute_inner_layout_rect(ctx.layout_rect);

        ctx.rectangle_fill(layout, Color::from_hex("ff808003"));

        self.auto_layout.reset_layout(layout);
        for child in self.children.iter_mut() {
            child.recompute_layout(layout, game_ctx);
            ctx.layout_rect = self.auto_layout.layout_child(layout, child.layout_component());
            child.render(&(), ctx, game_ctx);
        }

        ctx.layout_rect = copy;
    }

    fn input(&mut self, _state: &mut Self::State, evt: &crate::engine::input::InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        for child in self.children.iter_mut() {
            child.input(&mut (), evt, ctx)?;
        };
        return ControlFlow::Continue(())
    }

}

pub(crate) trait AutoLayout {

    fn reset_layout(&mut self, layout_rect: [f64; 4]);

    fn layout_child(&mut self, layout_rect: [f64; 4], layout: &mut LayoutComponent) -> [f64; 4];

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
        self.rect[1] += child[3] + self.gap;
        return self.rect;
    }

}