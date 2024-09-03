use std::any::TypeId;
use std::time::Instant;

#[derive(PartialEq)]
pub enum StateType {
    IDLE,
    MOVE,
}

pub struct State {
    pub state: StateType,
}

pub struct Hunger {
    pub value: u8,
    pub last_updated: Instant,
}
impl Hunger {
    pub fn new() -> Self {
        Self {
            value: 0,
            last_updated: Instant::now(),
        }
    }
}

pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

pub struct Shape {
    pub width: f32,
    pub height: f32,
    pub color: (u8, u8, u8, u8),
}
impl Shape {
    pub fn new(width: f32, height: f32, color: (u8, u8, u8, u8)) -> Self {
        Self {
            width,
            height,
            color,
        }
    }
}

pub struct Movement {
    pub distance: f32,
    pub destination_x: f32,
    pub destination_y: f32,
}

impl Movement {
    pub fn new() -> Self {
        Self {
            distance: 0.0,
            destination_x: 0.0,
            destination_y: 0.0,
        }
    }
}

pub struct Food {
    pub type_id: TypeId,
}
pub struct Stone {
    pub type_id: TypeId,
}
pub struct Wood {
    pub type_id: TypeId,
}
