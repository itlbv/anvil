use hecs::Entity;
use std::collections::HashMap;

use crate::components::Position;
use crate::{behaviors, BehaviorList, Knowledge};
use hecs::World as ComponentRegistry;

#[derive(PartialEq)]
pub enum EntityCommandType {
    MoveToPosition,
    RemoveFromMap,
}

pub struct EntityCommand {
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
    while !entity_commands.is_empty() {
        let entity_command = entity_commands.pop().unwrap();

        match entity_command.command_type {
            EntityCommandType::MoveToPosition => {
                // dispatch MoveToPosition for an entity
                let entity_behaviors = behaviors.get_mut(&entity_command.entity).unwrap();
                entity_behaviors.insert(0, behaviors::move_to_position());
                // add info to knowledge
                let knowledge = knowledges.get_mut(&entity_command.entity).unwrap();
                knowledge.destination_x = entity_command.param["x"].parse::<f32>().unwrap();
                knowledge.destination_y = entity_command.param["y"].parse::<f32>().unwrap();
            }
            EntityCommandType::RemoveFromMap => {
                registry
                    .remove_one::<Position>(entity_command.entity)
                    .expect("TODO: panic message");
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
    })
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
    })
}
