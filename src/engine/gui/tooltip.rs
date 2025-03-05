use std::hash::{Hash, Hasher};

use graphics::{image, CharacterCache, Transformed};
use ::image::ImageReader;

use crate::{engine::{render::RenderContext, scene::Update, spritesheet::Spritesheet, Color}, game::action::{Affliction, DamageType, Infliction}, GameContext};

use super::GUINode;

pub struct TooltipOverlay {

}


impl TooltipOverlay {

    pub fn new() -> Self {
        Self {  }
    }

}

impl GUINode for TooltipOverlay {

    fn update(&mut self, update: &Update, ctx: &mut crate::GameContext) {
        if let Some(tuple) = &mut ctx.tooltips.current_tooltip {
            tuple.3 += update.delta_time;
        }
    }

    fn render(&mut self, ctx: &mut RenderContext, game_ctx: &mut GameContext) {

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
                let dims = line.dims(ctx);
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


            let mut pos = [position[0] + 6., position[1] + 12.];
            for line in tooltip.lines.iter() {
                line.render(pos, ctx, game_ctx);
                let dims = line.dims(ctx);
                pos[1] += dims[1];
            }

        }
    }
}

pub struct TooltipRegistry {
    current_tooltip: Option<(u64, Tooltip, [f64; 2], f64)>,
}

impl TooltipRegistry {

    pub fn new() -> Self {
        Self { 
            current_tooltip: None
        }
    }
    
    pub fn show_delayed_prehash(&mut self, hash: u64, tooltip: &Tooltip, position: [f64; 2]) {
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

    pub fn hide_prehash(&mut self, hash: u64) {
        if let Some((current_hash, _, _, _)) = &self.current_tooltip {
            if hash == *current_hash {
                self.current_tooltip = None;
            }
        }
    }

}


#[derive(Clone, Hash)]
pub struct Tooltip {
    lines: Vec<TooltipLine>
}

impl Tooltip {

    pub fn new(title: String) -> Self {
        Self { lines: vec!(TooltipLine::Title(title)) }
    }

    pub fn add_line(&mut self, line: TooltipLine) {
        self.lines.push(line);
    }

}


#[derive(Clone)]
pub enum TooltipLine {
    Title(String),
    Body(String),
    ApCost(u16),
    Damage(DamageType),
    Inflicts(Infliction)
}

impl TooltipLine {

    fn dims(&self, ctx: &mut RenderContext) -> [f64; 2] {
        
        match &self {
            Self::Title(title) => [ctx.small_font.width(5, &title).unwrap_or(0.), 8.],
            Self::Body(body) => [ctx.small_font.width(5, &body).unwrap_or(0.), 8.],
            Self::ApCost(_ap_cost) => [8., 8.],
            Self::Damage(damage) => {
                let damage = match damage {
                    DamageType::Fixed(dmg) => dmg,
                    DamageType::FromWeapon(dmg) => dmg,
                };
                let mut lines = 0;
                if damage.slashing > 0. {
                    lines += 1;
                }
                if damage.piercing > 0. {
                    lines += 1;
                }
                if damage.bludgeoning > 0. {
                    lines += 1;
                }
                return [8., 8. * lines as f64]
            }
            _ => [8., 8.]
        }
    }

    fn render(&self, mut pos: [f64; 2], ctx: &mut RenderContext, _game_ctx: &mut GameContext) {
        match &self {
            Self::Title(title) => ctx.text_small(&title, 5, pos, Color::from_hex("ffffff")),
            Self::Body(body) => ctx.text_small(&body, 5, pos, Color::from_hex("5a6069")),
            Self::ApCost(ap_cost) => ctx.text_small(&format!("{ap_cost} AP"), 5, pos, Color::from_hex("446d99")),
            Self::Damage(damage) => {
                let damage = match damage {
                    DamageType::Fixed(dmg) => dmg,
                    DamageType::FromWeapon(dmg) => dmg,
                };
                if damage.slashing > 0. {
                    ctx.text_small(&format!("{} slashing", damage.slashing), 5, pos, Color::from_hex("5a6069"));
                    pos[1] += 8.;
                }
                if damage.piercing > 0. {
                    ctx.text_small(&format!("{} piercing", damage.slashing), 5, pos, Color::from_hex("5a6069"));
                    pos[1] += 8.;
                }
                if damage.bludgeoning > 0. {
                    ctx.text_small(&format!("{} bludgeoning", damage.slashing), 5, pos, Color::from_hex("5a6069"));
                }
            }
            Self::Inflicts(inflicts) => {
                let text = match &inflicts.affliction {
                    Affliction::Bleeding { duration } => format!("Target is Bleeding for {duration} turns"),
                    Affliction::Poisoned { duration } => format!("Target is Poisoned for {duration} turns"),
                    Affliction::Stunned { duration } => format!("Target is Stunned for {duration} turns"),
                };
                ctx.text_small(&text, 5, pos, Color::from_hex("5a6069"))
            }
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