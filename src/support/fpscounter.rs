// use crate::support::*;
use std::time::Duration;
use std::time::Instant;

pub struct FpsCounter {
    last: Instant,
    frame_count: u32,
    accumulator: Duration,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            last: Instant::now(),
            frame_count: 0,
            accumulator: Duration::ZERO,
        }
    }

    pub fn tick(&mut self) -> Option<f32> {
        let now = Instant::now();
        let delta = now - self.last;
        self.last = now;

        self.accumulator += delta;
        self.frame_count += 1;

        if self.accumulator >= Duration::from_secs(1) {
            let fps = self.frame_count as f32 / self.accumulator.as_secs_f32();
            self.accumulator = Duration::ZERO;
            self.frame_count = 0;
            Some(fps)
        } else {
            None
        }
    }
}
