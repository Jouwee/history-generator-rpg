use crate::{engine::{COLOR_BLACK, COLOR_WHITE}, globals::perf::perf, Actor, Color, GameContext, InputEvent, RenderContext, Update};

pub(crate) struct HeadsUpDisplay {
    health: NumberedBar,
    action_points: NumberedBar,
    stamina: NumberedBar,
}

impl HeadsUpDisplay {

    pub(crate) fn new() -> Self {
        return Self {
            health: (1., 1., None),
            action_points: (1., 1., None),
            stamina: (1., 1., None),
        }
    }

    pub(crate) fn preview_action_points(&mut self, player: &Actor, ap_cost: i32) {
        let preview = (player.ap.action_points as f64 - ap_cost as f64) / player.ap.max_action_points as f64;
        self.action_points.2 = Some(preview.max(0.));
    }

    pub(crate) fn clear_preview_action_points(&mut self) {
        self.action_points.2 = None;
    }

    pub(crate) fn render(&mut self, player: &Actor, ctx: &mut RenderContext, game_ctx: &mut GameContext) {
        perf().start("hud");
        ctx.image(&"gui/hud/background.png", [16, 16]);

        Self::draw_bar(ctx, &self.health, Color::from_hex("882309"), Color::from_hex("6c1307"), [42 + 16, 12 + 16, 182-42, 8]);
        Self::draw_bar(ctx, &self.action_points, Color::from_hex("77a8c8"), Color::from_hex("486a75"), [45 + 16, 23 + 16, 165-45, 4]);
        Self::draw_bar(ctx, &self.stamina, Color::from_hex("7b8c48"), Color::from_hex("566639"), [42 + 16, 30 + 16, 156-42, 4]);

        player.render_layers([14., 11.], ctx, game_ctx);

        let health_text = format!("{:.0} / {:.0}", player.hp.health_points(), player.hp.max_health_points());
        ctx.text(&health_text, game_ctx.assets.font_standard(), [42+16+8, 12+16+7+1], &COLOR_BLACK);
        ctx.text(&health_text, game_ctx.assets.font_standard(), [42+16+8, 12+16+7], &COLOR_WHITE);

        ctx.image(&"gui/hud/foreground.png", [16, 16]);
        perf().end("hud");
    }

    fn draw_bar(ctx: &mut RenderContext, bar: &NumberedBar, color_1: Color, color_2: Color, rect: [u32; 4]) {
        let x = rect[0] as f64;
        let y = rect[1] as f64;
        let w1 = rect[2] as f64 * bar.1;
        let w2 = rect[2] as f64 * bar.0;
        let h = rect[3] as f64;
        let hh = h / 2.;
        ctx.rectangle_fill([x, y, w1, h], COLOR_WHITE);
        ctx.rectangle_fill([x, y, w2, hh], color_1);
        ctx.rectangle_fill([x, y + hh, w2, hh], color_2);
        if let Some(preview) = bar.2 {
            let w3 = rect[2] as f64 * preview;
            let x = x + w3;
            let w3 = w2 - w3;
            ctx.rectangle_fill([x, y, w3, h], COLOR_WHITE.alpha(0.2));
        }
    }

    pub(crate) fn update(&mut self, player: &Actor, _update: &Update, _ctx: &mut GameContext) {
        self.health.0 = (player.hp.health_points() / player.hp.max_health_points()) as f64;
        self.action_points.0 = (player.ap.action_points as f64 / player.ap.max_action_points as f64) as f64;
        self.stamina.0 = (player.stamina.stamina / player.stamina.max_stamina) as f64;

        update_bar(&mut self.health);
        update_bar(&mut self.action_points);
        update_bar(&mut self.stamina);

    }

    pub(crate) fn input(&mut self, _player: &Actor, _evt: &InputEvent, _ctx: &mut GameContext) {
    }

}

type NumberedBar = (f64, f64, Option<f64>);

fn update_bar(bar: &mut NumberedBar) {
    let diff = bar.1 - bar.0;
    if diff <= 0. {
        bar.1 = bar.0;
    } else {
        bar.1 = bar.1 - (diff * 0.1).max(0.005);
    }
}