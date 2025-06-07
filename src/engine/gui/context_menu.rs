use piston::MouseButton;

use crate::{engine::gui::{layout_component::LayoutComponent, InputResult, UINode}, Color, GameContext, InputEvent, RenderContext};

const ROW_HEIGHT: f64 = 11.;

pub(crate) struct ContextMenu {
    layout: LayoutComponent,
    hover_index: Option<usize>
}

impl ContextMenu {

    pub(crate) fn new() -> Self {
        let mut layout = LayoutComponent::new();
        layout.size([48., 24.]).padding([1.; 4]);
        Self {
            layout,
            hover_index: None
        }
    }

    fn option_idx_from_pos(&self, cursor: &[f64; 2]) -> Option<usize> {
        if self.layout.hitbox(cursor) {
            let i = (cursor[1] - self.layout.last_layout[1]) / ROW_HEIGHT;
            return Some(i as usize);
        } else {
            return None;
        }
    }

}

impl UINode for ContextMenu {
    type State = ContextMenuModel;
    type Input = (i32, String);

    fn layout_component(&mut self) -> &mut LayoutComponent {
        return &mut self.layout
    }

    fn init(&mut self, state: &Self::State, _game_ctx: &mut GameContext) {
        let _ = self.layout.size([64. + 2., state.items.len() as f64 * ROW_HEIGHT + 2.]);
    }

    fn render(&mut self, state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        let copy = ctx.layout_rect;
        let full_rect = self.layout.compute_layout_rect(ctx);
        ctx.rectangle_fill(full_rect, Color::from_hex("090714"));
        ctx.layout_rect = self.layout.compute_inner_layout_rect(ctx);
        ctx.rectangle_fill(ctx.layout_rect, Color::from_hex("24232a"));
        let hover_i = self.hover_index.unwrap_or(9999);
        let mut y = 0;
        for (i, (_id, item)) in state.items.iter().enumerate() {
            if hover_i == i {
                ctx.rectangle_fill([ctx.layout_rect[0], ctx.layout_rect[1] + y as f64, ctx.layout_rect[2], ROW_HEIGHT], Color::from_hex("35394a"));
            }
            ctx.text_shadow(item, game_ctx.assets.font_standard(), [ctx.layout_rect[0] as i32 + 4, ctx.layout_rect[1] as i32 + y + 9], &Color::from_hex("ffffff"));
            y += ROW_HEIGHT as i32;
        }
        ctx.layout_rect = copy;
    }

    fn input(&mut self, _state: &mut Self::State, evt: &InputEvent, _game_ctx: &mut GameContext) -> InputResult<Self::Input> {
        match evt {
            InputEvent::Click { button: MouseButton::Left, pos } => {
                if let Some(i) = self.option_idx_from_pos(pos) {
                    if let Some(v) = _state.items.get(i) {
                        return InputResult::Consume(v.clone());
                    }
                }
            },
            InputEvent::MouseMove { pos } => self.hover_index = self.option_idx_from_pos(pos),
            _ => ()
        }
        return InputResult::None;
    }

}

pub(crate) struct ContextMenuModel {
    pub(crate) items: Vec<(i32, String)>
}