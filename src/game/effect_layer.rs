use std::f64::consts::PI;

use graphics::{image, Transformed};

use crate::{commons::interpolate::{lerp, Interpolate}, engine::{assets::{assets, ImageSheetAsset}, geometry::Coord2, render::RenderContext, scene::Update, Palette, COLOR_BLACK}, GameContext, SPRITE_FPS};

pub(crate) struct EffectLayer {
    damage_numbers: Vec<DamageNumber>,
    projectiles: Vec<Projectile>,
    sprites: Vec<Sprite>,
}

impl EffectLayer {

    pub(crate) fn new() -> EffectLayer {
        EffectLayer {
            damage_numbers: Vec::new(),
            projectiles: Vec::new(),
            sprites: Vec::new(),
        }
    }

    pub(crate) fn render(&mut self, ctx: &mut RenderContext) {
        let mut assets = assets();
        let font = assets.font_standard();
        for dn in self.damage_numbers.iter_mut() {
            let mut pos = [dn.pos.x as f64 * 24., dn.pos.y as f64 * 24.];
            // Center text
            if dn.width == 0. {
                dn.width = font.width(&dn.text);
            }
            pos[0] += 12. - (dn.width / 2.);
            // Animate upwards - Ease out
            pos[1] = lerp(pos[1], pos[1] - 24., Interpolate::EaseOutSine.interpolate(dn.lifetime));
            let pos = [pos[0] as i32, pos[1] as i32];
            // black border - the stupid way
            {
                let mut lpos = pos;
                lpos[0] -= 1;
                ctx.text(&dn.text, font, lpos, &COLOR_BLACK);
                let mut lpos = pos;
                lpos[0] += 1;
                ctx.text(&dn.text, font, lpos, &COLOR_BLACK);
                let mut lpos = pos;
                lpos[1] -= 1;
                ctx.text(&dn.text, font, lpos, &COLOR_BLACK);
                let mut lpos = pos;
                lpos[1] += 1;
                ctx.text(&dn.text, font, lpos, &COLOR_BLACK);
            }
            // actual text
            ctx.text(&dn.text, font, pos, &dn.color.color());
        }

        for projectile in self.projectiles.iter_mut() {
            let sheet = assets.image_sheet(&projectile.sprite.path, projectile.sprite.tile_size.clone());

            let pct = projectile.lifetime / projectile.duration;
            let x = lerp(projectile.from.x as f64, projectile.to.x as f64, pct);
            let y = lerp(projectile.from.y as f64, projectile.to.y as f64, pct);

            let copy: [[f64; 3]; 2] = ctx.context.transform;
            let angle_degrees = f64::atan2((projectile.to.y - projectile.from.y) as f64, (projectile.to.x - projectile.from.x) as f64) * 180. / PI;
            let pos = [x * 24. + 12., y * 24. + 12.];
            let transform = ctx.context.transform.trans(pos[0], pos[1]).rot_deg(angle_degrees);
            let sprite_index = ((projectile.lifetime / SPRITE_FPS) as usize) % sheet.len();
            image(sheet.get(sprite_index).unwrap(), transform, ctx.gl);
            ctx.context.transform = copy;
        }

        for sprite in self.sprites.iter_mut() {
            let sheet = assets.image_sheet(&sprite.sprite.path, sprite.sprite.tile_size.clone());
            let sprite_index = (sprite.lifetime / SPRITE_FPS) as usize;
            if sprite_index >= sheet.len() {
                sprite.done = true;
            } else {
                ctx.texture(sheet.get(sprite_index).unwrap(), ctx.at(sprite.pos.x as f64 * 24. + 12. - (sheet.tile_size.0 as f64 / 2.), sprite.pos.y as f64 * 24. + 12. - (sheet.tile_size.1 as f64 / 2.)));
            }
        }
        self.sprites.retain(|n| !n.done);


    }

    pub(crate) fn update(&mut self, update: &Update, _ctx: &mut GameContext) {
        for damage_number in self.damage_numbers.iter_mut() {
            damage_number.lifetime = damage_number.lifetime + update.delta_time;
        }
        self.damage_numbers.retain(|n| n.lifetime < 1.);
        for projectile in self.projectiles.iter_mut() {
            projectile.lifetime = projectile.lifetime + update.delta_time;
        }
        self.projectiles.retain(|n| n.lifetime < n.duration);
        for sprite in self.sprites.iter_mut() {
            sprite.lifetime = sprite.lifetime + update.delta_time;
        }
    }

    pub(crate) fn add_damage_number(&mut self, pos: Coord2, damage: f32) {
        self.add_text_indicator(pos, &format!("{:.0}", damage), Palette::Red);
    }

    pub(crate) fn add_text_indicator(&mut self, pos: Coord2, text: &str, color: Palette) {
        self.damage_numbers.push(DamageNumber {
            pos,
            width: 0.,
            text: String::from(text),
            color,
            lifetime: 0.
        });
    }

    pub(crate) fn add_projectile(&mut self, from: Coord2, to: Coord2, speed: f64, sprite: ImageSheetAsset) {
        let duration = from.dist(&to) as f64 / speed;
        self.projectiles.push(Projectile {
            from,
            to,
            duration,
            lifetime: 0.,
            sprite
        })
    }

    pub(crate) fn play_sprite(&mut self, pos: Coord2, sprite: ImageSheetAsset) {
        self.sprites.push(Sprite { pos, sprite, lifetime: 0., done: false });
    }

}

struct DamageNumber {
    pos: Coord2,
    width: f64,
    text: String,
    color: Palette,
    lifetime: f64
}

struct Projectile {
    from: Coord2,
    to: Coord2,
    duration: f64,
    lifetime: f64,
    sprite: ImageSheetAsset,
}

struct Sprite {
    pos: Coord2,
    sprite: ImageSheetAsset,
    done: bool,
    lifetime: f64,
}