use std::f64::consts::PI;

use graphics::CharacterCache;

use crate::{commons::rng::Rng, engine::{geometry::{Coord2, Vec2}, render::RenderContext, scene::Update, Color, Palette}, GameContext};

pub struct EffectLayer {
    damage_numbers: Vec<DamageNumber>
}

impl EffectLayer {

    pub fn new() -> EffectLayer {
        EffectLayer {
            damage_numbers: Vec::new()
        }
    }

    pub fn render(&mut self, ctx: &mut RenderContext, _game_ctx: &GameContext) {
        for dn in self.damage_numbers.iter_mut() {
            let mut pos = [dn.pos.x as f64 * 24., dn.pos.y as f64 * 24.];
            // Center text
            if dn.width == 0. {
                dn.width = ctx.small_font.width(5, &dn.text).unwrap_or(0.);
            }
            pos[0] += 12. - (dn.width / 2.);
            // Animate upwards - Ease out
            pos[0] += dn.speed.x as f64 * f64::sin((dn.lifetime * PI) / 2.) * 16.;
            pos[1] += dn.speed.y as f64 * f64::sin((dn.lifetime * PI) / 2.) * 16.;
            // black border - the stupid way
            {
                let mut lpos = pos;
                lpos[0] -= 1.;
                ctx.text_small(&dn.text, 5, lpos, Color::from_hex("000000"));
                let mut lpos = pos;
                lpos[0] += 1.;
                ctx.text_small(&dn.text, 5, lpos, Color::from_hex("000000"));
                let mut lpos = pos;
                lpos[1] -= 1.;
                ctx.text_small(&dn.text, 5, lpos, Color::from_hex("000000"));
                let mut lpos = pos;
                lpos[1] += 1.;
                ctx.text_small(&dn.text, 5, lpos, Color::from_hex("000000"));
            }
            // actual text
            ctx.text_small(&dn.text, 5, pos, dn.color.color());
        }

    }

    pub fn update(&mut self, update: &Update, _ctx: &mut GameContext) {
        for damage_number in self.damage_numbers.iter_mut() {
            damage_number.lifetime = damage_number.lifetime + update.delta_time;
        }
        self.damage_numbers.retain(|n| n.lifetime < 1.);
    }

    pub fn add_damage_number(&mut self, pos: Coord2, damage: f32) {
        self.add_text_indicator(pos, &format!("{:.1}", damage), Palette::Red);
    }

    pub fn add_text_indicator(&mut self, pos: Coord2, text: &str, color: Palette) {
        let mut rng = Rng::rand();
        let speed = Vec2::xy(rng.randf() * 1.6 - 0.8, -3. + rng.randf());
        self.damage_numbers.push(DamageNumber {
            pos,
            speed,
            width: 0.,
            text: String::from(text),
            color,
            lifetime: 0.
        });
    }

}

struct DamageNumber {
    pos: Coord2,
    speed: Vec2,
    width: f64,
    text: String,
    color: Palette,
    lifetime: f64
}