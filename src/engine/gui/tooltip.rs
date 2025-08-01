use std::hash::{Hash, Hasher};

use graphics::{image, Transformed};
use ::image::ImageReader;

use crate::{engine::{assets::{assets, Font}, gui::{layout_component::LayoutComponent, UINode}, render::RenderContext, scene::Update, spritesheet::Spritesheet, Color, COLOR_WHITE}, GameContext};

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

            let spritesheet = ImageReader::open("./assets/sprites/gui/tooltip/tooltip.png").unwrap().decode().unwrap();
            let sprite = Spritesheet::new(spritesheet, (8, 8));

            // Corners
            let transform = ctx.context.transform.trans(position[0], position[1]);
            image(sprite.sprite(0, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0], position[1] + size[1] - 8.);
            image(sprite.sprite(0, 2), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1]);
            image(sprite.sprite(2, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + size[1] - 8.);
            image(sprite.sprite(2, 2), transform, ctx.gl);
            // Borders
            let transform = ctx.context.transform.trans(position[0] + 8., position[1]).scale((size[0]-16.) / 8., 1.);
            image(sprite.sprite(1, 0), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + 8., position[1] + size[1] - 8.).scale((size[0]-16.) / 8., 1.);
            image(sprite.sprite(1, 2), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0], position[1] + 8.).scale(1., (size[1]-16.) / 8.);
            image(sprite.sprite(0, 1), transform, ctx.gl);
            let transform = ctx.context.transform.trans(position[0] + size[0] - 8., position[1] + 8.).scale(1., (size[1]-16.) / 8.);
            image(sprite.sprite(2, 1), transform, ctx.gl);
            // Body
            let transform = ctx.context.transform.trans(position[0] + 8., position[1] + 8.).scale((size[0]-16.) / 8., (size[1]-16.) / 8.);
            image(sprite.sprite(1, 1), transform, ctx.gl);


            let mut pos = [position[0] as i32 + 6, position[1] as i32 + 12];
            for line in tooltip.lines.iter() {
                line.render(pos, ctx);
                let dims = line.dims(assets().font_standard());
                pos[1] += dims[1] as i32;
            }

        }
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

    pub(crate) fn new(title: String) -> Self {
        Self { lines: vec!(TooltipLine::Title(title)) }
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
