use std::time::{Duration, Instant};

use piston::{Button, ButtonState, Key};

use crate::{engine::{assets::assets, render::RenderContext, scene::Update, COLOR_BLACK, COLOR_WHITE}, game::InputEvent, globals::perf::perf};

pub(crate) struct DebugOverlay {
    active: bool,
    fps: CountingBlocks,
    render_time: TrackingBlocks,
    tps: CountingBlocks,
    update_time: TrackingBlocks,
    input_time: TrackingBlocks,
}

impl DebugOverlay {

    pub(crate) fn new() -> DebugOverlay {
        DebugOverlay {
            active: false,
            fps: CountingBlocks::new(),
            render_time: TrackingBlocks::new(),
            tps: CountingBlocks::new(),
            update_time: TrackingBlocks::new(),
            input_time: TrackingBlocks::new(),
        }
    }

    pub(crate) fn render(&mut self, context: &mut RenderContext) {
        self.fps.count();
        if self.active {
            context.rectangle_fill([0., 0., 128., 36.], COLOR_BLACK.alpha(0.5));
            context.text(format!("FPS: {:.0} - {:.3} (Teoretical: {:.0})", self.fps.average(), self.render_time.average(), 1./self.render_time.average()).as_str(), assets().font_standard(), [0, 12], &COLOR_WHITE);
            context.text(format!("TPS: {:.0} - {:.3} (Teoretical: {:.0})", self.tps.average(), self.update_time.average(), 1./self.update_time.average()).as_str(), assets().font_standard(), [0, 20], &COLOR_WHITE);
            context.text(format!("Inp: {:.0}", self.input_time.average()).as_str(), assets().font_standard(), [0, 28], &COLOR_WHITE);
            context.text(format!("Assets: {}", assets().asset_count()).as_str(), assets().font_standard(), [0, 36], &COLOR_WHITE);

            let perf_lines = perf().debug_lines();
            let mut y = 44.;
            for line in perf_lines {
                context.rectangle_fill([0., y, 128., 8.], COLOR_BLACK.alpha(0.5));
                context.text(&line, assets().font_standard(), [0, y as i32+6], &COLOR_WHITE);
                y += 8.
            }

        }
    }

    pub(crate) fn update(&mut self, _update: &Update) {
        self.tps.count();
    }

    pub(crate) fn render_time(&mut self, time: Duration) {
        self.render_time.add(time.as_secs_f64());
    }

    pub(crate) fn update_time(&mut self, time: Duration) {
        self.update_time.add(time.as_secs_f64());
    }

    pub(crate) fn input_time(&mut self, time: Duration) {
        self.input_time.add(time.as_secs_f64());
    }

    pub(crate) fn input(&mut self, input: &InputEvent) {
        if input.button_args.state == ButtonState::Press {
            if let Button::Keyboard(Key::F3) = input.button_args.button {
                self.active = !self.active;
            }
        }
    }
}

struct CountingBlocks {
    // Start, finish, count
    blocks: [(f64, usize); 10],
    start: Instant
}

impl CountingBlocks {
    
    fn new() -> CountingBlocks {
        CountingBlocks {
            blocks: [(0., 0); 10],
            start: Instant::now()
        }
    }

    fn count(&mut self) {
        self.blocks[0].1 += 1;
        let current = self.start.elapsed().as_secs_f64();
        if current - self.blocks[0].0 > 0.2 {
            // Shift-right
            for i in (0..9).rev() {
                self.blocks[i + 1] = self.blocks[i];
            }
            self.blocks[0] = (current, 0);
        }
    }

    fn average(&self) -> f64 {
        let mut sum = 0;
        for i in 0..10 {
            sum += self.blocks[i].1;
        }
        let current = self.start.elapsed().as_secs_f64();
        return sum as f64 / (current - self.blocks[9].0);
    }

}

struct TrackingBlocks {
    blocks: [f64; 10],
    idx: usize
}

impl TrackingBlocks {
    
    fn new() -> TrackingBlocks {
        TrackingBlocks {
            blocks: [0.; 10],
            idx: 0
        }
    }

    fn add(&mut self, v: f64) {
        self.blocks[self.idx] = v;
        self.idx = (self.idx + 1) % 10;
    }

    fn average(&self) -> f64 {
        let mut sum = 0.;
        for i in 0..10 {
            sum += self.blocks[i];
        }
        return sum as f64 / 10.;
    }

}