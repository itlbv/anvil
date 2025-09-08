use crate::time::{FixedDt, Tick};
use std::time::Instant;

pub struct SimLoop {
    pub fixed: FixedDt,
    accumulator: f32,
    last_real: Instant,
    pub tick: Tick,
    pub max_steps_per_frame: u32, // back-pressure guard
}

impl SimLoop {
    pub fn new(hz: u32) -> Self {
        Self {
            fixed: FixedDt::from_hz(hz),
            accumulator: 0.0,
            last_real: Instant::now(),
            tick: Tick(0),
            max_steps_per_frame: 8,
        }
    }

    /// Call once per frame. It returns how many fixed steps to process.
    pub fn begin_frame(&mut self) -> u32 {
        let now = Instant::now();
        let dt_real = (now - self.last_real).as_secs_f32();
        self.last_real = now;

        // Clamp to avoid spiral of death after long pauses.
        let dt_real = dt_real.min(self.fixed.seconds * self.max_steps_per_frame as f32);
        self.accumulator += dt_real;

        let mut steps = 0;
        while self.accumulator + 1e-9 >= self.fixed.seconds && steps < self.max_steps_per_frame {
            self.accumulator -= self.fixed.seconds;
            steps += 1;
        }
        steps
    }

    /// Advance simulation tick counter once per fixed step.
    pub fn advance_tick(&mut self) {
        self.tick = Tick(self.tick.0 + 1);
    }

    /// Alpha in [0,1) for render interpolation if you need it.
    pub fn alpha(&self) -> f32 {
        (self.accumulator / self.fixed.seconds).clamp(0.0, 1.0)
    }
}
