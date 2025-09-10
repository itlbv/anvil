use hecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::components::Position;
use crate::{behaviors, BehaviorList, Knowledge};
use hecs::World as ComponentRegistry;
use sdl2::ttf::init;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    MoveToPosition { x: f32, y: f32 },
    RemoveFromMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCommand {
    #[serde(with = "crate::entity_serde")]
    entity: Entity,
    kind: CommandType,
}

impl EntityCommand {
    pub fn move_to(entity: Entity, x: f32, y: f32) -> Self {
        Self {
            entity,
            kind: CommandType::MoveToPosition { x, y },
        }
    }

    pub fn remove_from_map(entity: Entity) -> Self {
        Self {
            entity,
            kind: CommandType::RemoveFromMap,
        }
    }
}

pub fn process_entity_commands(
    commands: &mut Vec<EntityCommand>,
    knowledges: &mut HashMap<Entity, Knowledge>,
    behaviors: &mut HashMap<Entity, BehaviorList>,
    registry: &mut ComponentRegistry,
) {
    while let Some(cmd) = commands.pop() {
        match cmd.kind {
            CommandType::MoveToPosition { x, y } => {
                let entity_behaviours = behaviors
                    .get_mut(&cmd.entity)
                    .expect("behaviours missing for entity");
                entity_behaviours.insert(0, behaviors::move_to_position());

                let knowledge = knowledges
                    .get_mut(&cmd.entity)
                    .expect("knowledg missing for entity");
                knowledge.destination_x = x;
                knowledge.destination_y = y;
            }
            CommandType::RemoveFromMap => {
                registry
                    .remove_one::<Position>(cmd.entity)
                    .expect("failed to remove Position component");
            }
        }
    }
}

pub fn push_new_command(
    entity_commands: &mut Vec<EntityCommand>,
    entity: Entity,
    command_kind: CommandType,
) {
    match command_kind {
        CommandType::MoveToPosition { x, y } => {
            entity_commands.push(EntityCommand {
                entity,
                kind: CommandType::MoveToPosition { x, y },
            });
        }
        CommandType::RemoveFromMap => {
            entity_commands.push(EntityCommand {
                entity,
                kind: CommandType::RemoveFromMap,
            });
        }
    }
}
