use std::time::Instant;

use piston::{Button, ButtonState, Key};

use crate::{engine::{render::RenderContext, scene::Update, Color}, game::InputEvent};

pub struct DebugOverlay {
    active: bool,
    fps: TrackingBlocks,
    tps: TrackingBlocks,
}

impl DebugOverlay {

    pub fn new() -> DebugOverlay {
        DebugOverlay {
            active: false,
            fps: TrackingBlocks::new(),
            tps: TrackingBlocks::new(),
        }
    }

    pub fn render(&mut self, context: &mut RenderContext) {
        self.fps.count();
        if self.active {
            context.rectangle_fill([0., 0., 128., 36.], Color::from_hex("00000080"));
            context.text(format!("FPS: {:.2}", self.fps.average()).as_str(), 11, [0., 16.], Color::from_hex("ffffff"));
            context.text(format!("TPS: {:.2}", self.tps.average()).as_str(), 11, [0., 34.], Color::from_hex("ffffff"));
        }
    }

    pub fn update(&mut self, _update: &Update) {
        self.tps.count();
    }

    pub fn input(&mut self, input: &InputEvent) {
        if input.button_args.state == ButtonState::Press {
            if let Button::Keyboard(Key::F3) = input.button_args.button {
                self.active = !self.active;
            }
        }
    }
}

struct TrackingBlocks {
    // Start, finish, count
    blocks: [(f64, usize); 10],
    start: Instant
}

impl TrackingBlocks {
    
    fn new() -> TrackingBlocks {
        TrackingBlocks {
            blocks: [(0., 0); 10],
            start: Instant::now()
        }
    }

    fn count(&mut self) {
        self.blocks[0].1 += 1;
        let current = self.start.elapsed().as_secs_f64();
        if current - self.blocks[0].0 > 1. {
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