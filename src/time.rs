#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Tick(pub u64);

#[derive(Clone, Copy, Debug)]
pub struct FixedDt {
    pub seconds: f32, // e.g., 1.0/60.0
}

impl FixedDt {
    pub const fn from_hz(hz: u32) -> Self {
        Self {
            seconds: 1.0 / (hz as f32),
        }
    }
}
