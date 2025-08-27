use crate::{engine::{assets::assets, geometry::Size2D, gui::{label::{HorizontalAlignment, Label}, layout_component::LayoutComponent, UIEvent, UINode}}, globals::perf::perf, world::date::WorldDate, GameContext, RenderContext};

pub(crate) struct TimeWidget {
    layout: LayoutComponent,
    current_time: WorldDate,
    label: Label,
}

impl TimeWidget {

    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([104., 16.]);
        return Self {
            layout,
            current_time: WorldDate::new(1, 1, 1),
            label: Label::text("").layout(|l| { l.size([104., 16.]).padding([0., 3., 0., 3.]); }).hor_alignment(HorizontalAlignment::Center)
        }
    }

    pub(crate) fn set_time(&mut self, time: &WorldDate) {
        if &self.current_time != time {
            self.current_time = time.clone();
            self.label.set_text(self.current_time.fmt_long());
        }
    }

}

impl UINode for TimeWidget {
    type State = ();
    type Input = UIEvent;

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout;
    }

    fn render(&mut self, _state: &(), ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("time_widget");

        let copy = ctx.layout_rect;
        ctx.layout_rect = self.layout_component().compute_inner_layout_rect(ctx.layout_rect);
            
        let rect = self.label.layout_component().compute_layout_rect(ctx.layout_rect);

        let sheet = assets().image_sheet("gui/title_holder.png", Size2D(32, 16));
        sheet.draw_as_scalable(rect, ctx);

        self.label.render(&(), ctx, game_ctx);

        ctx.layout_rect = copy;

        perf().end("time_widget");
    }

}