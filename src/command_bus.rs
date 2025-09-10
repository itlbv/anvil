use crate::entity_commands::EntityCommand;
use std::mem::swap;

pub struct CommandBus {
    pub incoming: Vec<EntityCommand>,
    pub processing: Vec<EntityCommand>,
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            incoming: Vec::new(),
            processing: Vec::new(),
        }
    }

    pub fn begin_tick(&mut self) {
        swap(&mut self.incoming, &mut self.processing);
        self.incoming.clear();
    }
}
