use std::{hash::{DefaultHasher, Hash, Hasher}, ops::ControlFlow};

use crate::{commons::damage_model::DamageRoll, engine::{assets::{assets, Font}, geometry::Size2D, gui::{layout_component::LayoutComponent, UINode}, input::InputEvent, render::RenderContext, scene::Update, Color, COLOR_WHITE}, GameContext};

pub(crate) struct TooltipOverlay {
    layout: LayoutComponent
}


impl TooltipOverlay {

    pub(crate) fn new() -> Self {
        Self { 
            layout: LayoutComponent::new()
        }
    }

}

impl UINode for TooltipOverlay {
    type State = ();
    type Input = ();
    
    fn layout_component(&mut self) -> &mut LayoutComponent {
        &mut self.layout
    }

    fn update(&mut self, _state: &mut Self::State, update: &Update, ctx: &mut crate::GameContext) {
        if let Some(tuple) = &mut ctx.tooltips.current_tooltip {
            tuple.3 += update.delta_time;
        }
    }

    fn render(&mut self, _state: &Self::State, ctx: &mut RenderContext, game_ctx: &mut GameContext) {

        let tooltip = match &game_ctx.tooltips.current_tooltip {
            Some(v) => Some(v.clone()),
            None => None
        };

        if let Some((_hash, tooltip, cursor, time)) = tooltip {
            if time < 0.5 {
                return
            }
            // Compute size
            let mut size = [10., 10.];
            for line in tooltip.lines.iter() {
                let dims = line.dims(assets().font_standard());
                size[0] = f64::max(size[0], dims[0] + 10.);
                size[1] += dims[1];
            }

            // Compute position
            let mut position = [cursor[0].round(), cursor[1].round()];
            position[0] -= size[0] * 0.5;
            position[1] -= size[1];

            let rect = [position[0], position[1], size[0], size[1]];
            let sheet = assets().image_sheet("gui/tooltip/tooltip.png", Size2D(8, 8));
            sheet.draw_as_scalable(rect, ctx);

            let mut pos = [position[0] as i32 + 6, position[1] as i32 + 12];
            for line in tooltip.lines.iter() {
                line.render(pos, ctx);
                let dims = line.dims(assets().font_standard());
                pos[1] += dims[1] as i32;
            }

        }
    }

    fn input(&mut self, _state: &mut Self::State, evt: &InputEvent, ctx: &mut GameContext) -> ControlFlow<Self::Input> {
        if let InputEvent::MouseMove { pos: _ } = evt {
            ctx.tooltips.hide();
        }
        return ControlFlow::Continue(());
    }
}

pub(crate) struct TooltipRegistry {
    current_tooltip: Option<(u64, Tooltip, [f64; 2], f64)>,
}

impl TooltipRegistry {

    pub(crate) fn new() -> Self {
        Self { 
            current_tooltip: None
        }
    }

    pub(crate) fn show_delayed(&mut self, tooltip: &Tooltip, position: [f64; 2]) {
        let mut hasher = DefaultHasher::new();
        tooltip.hash(&mut hasher);
        let hash = hasher.finish();
        self.show_delayed_prehash(hash, tooltip, position);
    }
    
    pub(crate) fn show_delayed_prehash(&mut self, hash: u64, tooltip: &Tooltip, position: [f64; 2]) {
        match &mut self.current_tooltip {
            Some(tuple) => {
                if hash != tuple.0 {
                    self.current_tooltip = Some((hash, tooltip.clone(), position, 0.));
                } else {
                    // If moved mouse
                    if position != tuple.2 && tuple.3 < 1. {
                        // Only refreshes position and timer
                        tuple.2 = position;
                        tuple.3 = 0.;
                    }
                }
            },
            None => {
                self.current_tooltip = Some((hash, tooltip.clone(), position, 0.));
            }
        }
    }

    pub(crate) fn hide(&mut self) {
        self.current_tooltip = None;
    }

    pub(crate) fn hide_prehash(&mut self, hash: u64) {
        if let Some((current_hash, _, _, _)) = &self.current_tooltip {
            if hash == *current_hash {
                self.current_tooltip = None;
            }
        }
    }

}


#[derive(Clone, Hash)]
pub(crate) struct Tooltip {
    lines: Vec<TooltipLine>
}

impl Tooltip {

    pub(crate) fn new(title: &str) -> Self {
        Self { lines: vec!(TooltipLine::Title(String::from(title))) }
    }

    pub(crate) fn add_line(&mut self, line: TooltipLine) {
        self.lines.push(line);
    }

}


#[derive(Clone)]
pub(crate) enum TooltipLine {
    Title(String),
    Body(String),
    ApCost(u16),
    StaminaCost(f32),
    DamageRoll(DamageRoll),
}

impl TooltipLine {

    fn dims(&self, font: &mut Font) -> [f64; 2] {
        match &self {
            Self::Title(title) => [font.width(&title), 8.],
            Self::Body(body) => [font.width(&body), 8.],
            Self::ApCost(_ap_cost) => [8., 8.],
            _ => [8., 8.]
        }
    }

    fn render(&self, pos: [i32; 2], ctx: &mut RenderContext) {
        match &self {
            Self::Title(title) => ctx.text(&title, assets().font_standard(), pos, &COLOR_WHITE),
            Self::DamageRoll(roll) => ctx.text(&roll.to_string(), assets().font_standard(), pos, &COLOR_WHITE),
            Self::Body(body) => ctx.text(&body, assets().font_standard(), pos, &Color::from_hex("5a6069")),
            Self::ApCost(ap_cost) => ctx.text(&format!("{ap_cost} AP"), assets().font_standard(), pos, &Color::from_hex("446d99")),
            Self::StaminaCost(stamina_cost) => ctx.text(&format!("{stamina_cost} ST"), assets().font_standard(), pos, &Color::from_hex("88ae59")),
        }
    }

}

impl Hash for TooltipLine {

    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            Self::Title(title) => title.hash(state),
            Self::Body(title) => title.hash(state),
            _ => ()
        }
    }

}
