use hecs::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::components::Position;
use crate::{behaviors, BehaviorList, Knowledge};
use hecs::World as ComponentRegistry;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityCommandType {
    MoveToPosition,
    RemoveFromMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCommand {
    #[serde(with = "crate::entity_serde")]
    entity: Entity,
    command_type: EntityCommandType,
    param: HashMap<String, String>,
}

pub fn process_entity_commands(
    entity_commands: &mut Vec<EntityCommand>,
    knowledges: &mut HashMap<Entity, Knowledge>,
    behaviors: &mut HashMap<Entity, BehaviorList>,
    registry: &mut ComponentRegistry,
) {
    while let Some(entity_command) = entity_commands.pop() {
        match entity_command.command_type {
            EntityCommandType::MoveToPosition => {
                let entity_behaviors = behaviors
                    .get_mut(&entity_command.entity)
                    .expect("behaviors missing for entity");
                entity_behaviors.insert(0, behaviors::move_to_position());

                // Update knowledge with target destination
                let knowledge = knowledges
                    .get_mut(&entity_command.entity)
                    .expect("knowledge missing for entity");
                knowledge.destination_x = entity_command.param["x"].parse::<f32>().unwrap();
                knowledge.destination_y = entity_command.param["y"].parse::<f32>().unwrap();
            }
            EntityCommandType::RemoveFromMap => {
                registry
                    .remove_one::<Position>(entity_command.entity)
                    .expect("failed to remove Position component");
            }
        }
    }
}

pub fn push_new_command(
    entity_commands: &mut Vec<EntityCommand>,
    entity: Entity,
    command_type: EntityCommandType,
) {
    entity_commands.push(EntityCommand {
        entity,
        command_type,
        param: HashMap::new(),
    });
}

pub fn push_new_command_with_param(
    entity_commands: &mut Vec<EntityCommand>,
    entity: Entity,
    command_type: EntityCommandType,
    param: HashMap<String, String>,
) {
    entity_commands.push(EntityCommand {
        entity,
        command_type,
        param,
    });
}
